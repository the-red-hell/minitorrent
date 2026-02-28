use ::core::net::SocketAddrV4;
use core::fmt::Write as _;
use embedded_io_async::{Read, Write};
use embedded_nal_async::Dns;
use heapless::string::String;

use crate::{
    BitTorrenter, BitTorrenterError, MetaInfoFile, core::tracker::TrackerRequest, fs::VolumeMgr,
};

// ============================================================================
// TcpConnector Trait
// ============================================================================

/// A trait for establishing TCP connections where the **caller provides buffers**.
///
/// # Motivation
///
/// Unlike `embedded_nal_async::TcpConnect`, this trait accepts mutable buffer
/// references as parameters. This design avoids interior mutability (RefCell,
/// Mutex) in the network implementation, which is important for:
///
/// - **Embedded systems**: No runtime overhead from borrow checking or locking
/// - **Clear ownership**: The caller (e.g., `BitTorrenter`) owns the buffers
/// - **Flexible buffer sizes**: Different callers can provide different buffer sizes
///
/// # Buffer Lifetimes
///
/// The returned `Connection` borrows from the buffers, so the connection cannot
/// outlive them. This is enforced at compile time.
///
/// # Example
///
/// ```ignore
/// let mut rx = [0u8; 4096];
/// let mut tx = [0u8; 1024];
/// let socket = connector.connect(addr, &mut rx, &mut tx).await?;
/// // socket borrows rx and tx - they cannot be used until socket is dropped
/// ```
///
/// # Note on `async fn` in traits
///
/// We use `async fn` directly here because this trait is designed for embedded
/// single-threaded executors (embassy) where `Send` bounds are not required.
#[allow(async_fn_in_trait)]
pub trait TcpConnector {
    /// The error type returned when a connection fails.
    ///
    /// Must implement `Debug` for error reporting.
    type Error: core::fmt::Debug;

    /// The established TCP connection type.
    ///
    /// This type must implement `embedded_io_async::Read` and `Write` for
    /// bidirectional communication.
    type Connection<'a>: Read<Error = Self::Error> + Write<Error = Self::Error>
    where
        Self: 'a;

    /// Establish a TCP connection to the given remote address.
    ///
    /// # Arguments
    ///
    /// * `remote` - The socket address (IP + port) to connect to
    /// * `rx_buffer` - Buffer for incoming data (size determines max receive window)
    /// * `tx_buffer` - Buffer for outgoing data (size determines max send window)
    ///
    /// # Returns
    ///
    /// A connected socket that borrows the provided buffers, or an error if
    /// the connection could not be established.
    async fn connect<'a>(
        &'a self,
        remote: SocketAddrV4,
        rx_buffer: &'a mut [u8],
        tx_buffer: &'a mut [u8],
    ) -> Result<Self::Connection<'a>, Self::Error>;
}

use url::SimpleUrl;

mod url {
    //! A simple URL parser for embedded systems without atomic operations (`url` uses `fetch_add`)
    use heapless::String;

    #[derive(Debug, Clone)]
    pub struct SimpleUrl<'a> {
        scheme: &'a str,
        host: &'a str,
        port: Option<u16>,
        path: &'a str,
        query: Option<String<512>>,
    }

    impl<'a> SimpleUrl<'a> {
        /// Parse a URL string into components
        pub fn parse(url: &'a str) -> Result<Self, &'static str> {
            // Parse scheme (e.g., "http://")
            let (scheme, rest) = url.split_once("://").ok_or("Invalid URL: missing scheme")?;

            // Parse host and optional port
            let (host_port, path_query) = if let Some(pos) = rest.find('/') {
                (&rest[..pos], &rest[pos..])
            } else if let Some(pos) = rest.find('?') {
                // No path, but query exists: http://example.com?query
                (&rest[..pos], &rest[pos..])
            } else {
                // No path or query
                (rest, "")
            };

            let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
                let port_num = p.parse::<u16>().map_err(|_| "Invalid port number")?;
                (h, Some(port_num))
            } else {
                (host_port, None)
            };

            // Separate path from query
            let (path, query_str) = if path_query.is_empty() {
                ("/", None)
            } else if let Some(pos) = path_query.find('?') {
                let p = &path_query[..pos];
                let path = if p.is_empty() { "/" } else { p };
                (path, Some(&path_query[pos + 1..]))
            } else {
                (path_query, None)
            };

            let query = query_str.map(|q| {
                let mut s = String::<512>::new();
                s.push_str(q).ok();
                s
            });

            Ok(SimpleUrl {
                scheme,
                host,
                port,
                path,
                query,
            })
        }

        /// Get the host string
        pub fn host_str(&self) -> Option<&str> {
            Some(self.host)
        }

        /// Get the port, or default based on scheme
        pub fn port(&self) -> Option<u16> {
            self.port.or(match self.scheme {
                "http" => Some(80),
                "https" => Some(443),
                _ => None,
            })
        }

        /// Get the path
        pub fn path(&self) -> &str {
            self.path
        }

        /// Get the query string
        pub fn query(&self) -> Option<&str> {
            self.query.as_ref().map(|s| s.as_str())
        }

        /// Set the query string
        pub fn set_query(&mut self, query: Option<&str>) {
            self.query = query.map(|q| {
                let mut s = String::<512>::new();
                s.push_str(q).ok();
                s
            });
        }
    }
}

impl<NET, V, const RX: usize, const TX: usize> BitTorrenter<NET, V, RX, TX>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    /// Send a request to the BitTorrent tracker and receive the response.
    ///
    /// This performs an HTTP GET request to the tracker's announce URL with
    /// the required BitTorrent parameters (info_hash, peer_id, port, etc.).
    ///
    /// # Arguments
    ///
    /// * `metadata` - The parsed .torrent file containing the announce URL
    /// * `rx_buf` - Buffer to store the tracker's bencoded response
    ///
    /// # Returns
    ///
    /// The number of bytes written to `rx_buf` (the response body only,
    /// HTTP headers are stripped).
    pub async fn make_tracker_request(
        &mut self,
        metadata: &MetaInfoFile<'_>,
        rx_buf: &mut [u8],
    ) -> Result<usize, BitTorrenterError<NET, V>> {
        let mut url = SimpleUrl::parse(metadata.announce).expect("Could not parse URL");
        let tracker_request = TrackerRequest::new(
            &metadata.info_hash,
            &self.peer_id,
            self.port,
            metadata.info.length,
        );
        let query = tracker_request.to_url_encoded();
        url.set_query(Some(&query));
        let bytes_written = self.make_http_request(&url, rx_buf).await?;

        // Move the body of the HTTP response to the beginning of the buffer
        let body_start = http_header_end_pos(&rx_buf[..bytes_written]);
        rx_buf.copy_within(body_start..bytes_written, 0);
        Ok(bytes_written - body_start)
    }

    /// Perform an HTTP GET request and read the response.
    ///
    /// Uses the internal socket buffers owned by `BitTorrenter` for the TCP
    /// connection. The response (headers + body) is written to `rx_buf`.
    async fn make_http_request(
        &mut self,
        url: &SimpleUrl<'_>,
        rx_buf: &mut [u8],
    ) -> Result<usize, BitTorrenterError<NET, V>> {
        let host = url.host_str().unwrap_or_default();
        let port = url.port().unwrap_or(80);
        let path = url.path();

        // Resolve hostname to IP address using DNS (UDP-based, no buffers needed)
        let ip = self
            .net()
            .get_host_by_name(host, embedded_nal_async::AddrType::IPv4)
            .await
            .map_err(BitTorrenterError::DnsError)?;

        let ip = match ip {
            core::net::IpAddr::V4(ipv4) => ipv4,
            core::net::IpAddr::V6(_) => {
                unreachable!("IPv6 not supported in this application, we only use IPv4 trackers")
            }
        };

        // Connect to server using our owned socket buffers
        let mut tcp = self
            .net
            .connect(
                SocketAddrV4::new(ip, port),
                &mut self.socket_buffers.rx,
                &mut self.socket_buffers.tx,
            )
            .await
            .map_err(BitTorrenterError::TcpError)?;

        // Construct HTTP GET request
        let mut request = String::<512>::new();
        write!(
            request,
            "GET {}?{} HTTP/1.1\r\n",
            path,
            url.query()
                .expect("We set them when url-encoding the tracker-request")
        )
        .unwrap();
        write!(request, "Host: {}\r\n", host).unwrap();
        write!(request, "Connection: close\r\n").unwrap();
        write!(request, "\r\n").unwrap();

        // Send request
        tcp.write_all(request.as_bytes())
            .await
            .map_err(BitTorrenterError::TcpError)?;
        tcp.flush().await.map_err(BitTorrenterError::TcpError)?;

        // Read response
        tcp.read(rx_buf).await.map_err(BitTorrenterError::TcpError)
    }
}

fn http_header_end_pos(response: &[u8]) -> usize {
    // Find the end of the HTTP header (indicated by \r\n\r\n)
    if let Some(pos) = response.windows(4).position(|window| window == b"\r\n\r\n") {
        pos + 4
    } else {
        0 // If no header found, return the whole response
    }
}

pub fn percent_encode(bytes: &[u8]) -> String<60> {
    let mut encoded = String::<60>::new();
    for &b in bytes {
        write!(encoded, "%{:02X}", b).unwrap();
    }
    encoded
}

#[cfg(test)]
mod tests {
    use crate::core::{InfoHash, PeerId};

    use super::*;

    #[test]
    fn test_tracker_request_url_encoding() {
        let info_hash: InfoHash = [0u8; 20];
        let peer_id: PeerId = [1u8; 20];

        assert_eq!(
            percent_encode(&info_hash),
            "%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00"
        );
        assert_eq!(
            percent_encode(&peer_id),
            "%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01"
        );
    }

    #[test]
    fn test_simple_url_basic_http() {
        let url = SimpleUrl::parse("http://example.com").unwrap();
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.port(), Some(80));
        assert_eq!(url.path(), "/");
        assert_eq!(url.query(), None);
    }

    #[test]
    fn test_simple_url_with_port() {
        let url = SimpleUrl::parse("http://example.com:8080").unwrap();
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.port(), Some(8080));
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn test_simple_url_with_path() {
        let url = SimpleUrl::parse("http://example.com/path/to/resource").unwrap();
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.port(), Some(80));
        assert_eq!(url.path(), "/path/to/resource");
        assert_eq!(url.query(), None);
    }

    #[test]
    fn test_simple_url_with_query() {
        let url = SimpleUrl::parse("http://example.com?key=value&foo=bar").unwrap();
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.port(), Some(80));
        assert_eq!(url.path(), "/");
        assert_eq!(url.query(), Some("key=value&foo=bar"));
    }

    #[test]
    fn test_simple_url_with_path_and_query() {
        let url = SimpleUrl::parse("http://tracker.com:6969/announce?info_hash=test").unwrap();
        assert_eq!(url.host_str(), Some("tracker.com"));
        assert_eq!(url.port(), Some(6969));
        assert_eq!(url.path(), "/announce");
        assert_eq!(url.query(), Some("info_hash=test"));
    }

    #[test]
    fn test_simple_url_https_default_port() {
        let url = SimpleUrl::parse("https://secure.example.com").unwrap();
        assert_eq!(url.host_str(), Some("secure.example.com"));
        assert_eq!(url.port(), Some(443));
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn test_simple_url_set_query() {
        let mut url = SimpleUrl::parse("http://example.com/path").unwrap();
        assert_eq!(url.query(), None);

        url.set_query(Some("new=query&param=value"));
        assert_eq!(url.query(), Some("new=query&param=value"));

        url.set_query(None);
        assert_eq!(url.query(), None);
    }

    #[test]
    fn test_simple_url_invalid_no_scheme() {
        let result = SimpleUrl::parse("example.com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid URL: missing scheme");
    }

    #[test]
    fn test_simple_url_invalid_port() {
        let result = SimpleUrl::parse("http://example.com:invalid");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number");
    }

    #[test]
    fn test_simple_url_complex_query() {
        let url = SimpleUrl::parse(
            "http://tracker.example.com:6969/announce?info_hash=%01%02%03&peer_id=test123&port=6881"
        ).unwrap();
        assert_eq!(url.host_str(), Some("tracker.example.com"));
        assert_eq!(url.port(), Some(6969));
        assert_eq!(url.path(), "/announce");
        assert_eq!(
            url.query(),
            Some("info_hash=%01%02%03&peer_id=test123&port=6881")
        );
    }
}
