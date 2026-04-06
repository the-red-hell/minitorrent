use ::core::fmt::Write as _;
use ::core::net::SocketAddrV4;
use embedded_io_async::{Read, Write};
use embedded_nal_async::Dns;
use heapless::string::String;

use crate::{
    BitTorrenter, BitTorrenterError, MetaInfoFile, TcpConnector,
    bittorrenter::states::{Downloading, RequestingTracker},
    core::tracker::{TrackerRequest, TrackerResponse},
    fs::VolumeMgr,
    net::url::SimpleUrl,
};

impl<NET, V, const RX: usize, const TX: usize> BitTorrenter<NET, V, RequestingTracker, RX, TX>
where
    NET: TcpConnector + Dns,
    V: VolumeMgr,
{
    pub async fn into_downloader(
        mut self,
        metainfo: &MetaInfoFile<'_>,
        rx_buf: &mut [u8],
    ) -> Result<BitTorrenter<NET, V, Downloading, RX, TX>, BitTorrenterError<NET, V>> {
        #[cfg(feature = "defmt")]
        defmt::trace!(
            "Requesting tracker with info_hash: {:x}",
            metainfo.info_hash
        );
        #[cfg(feature = "log")]
        log::trace!(
            "Requesting tracker with info_hash: {:x?}",
            metainfo.info_hash
        );
        let bytes_written = self.make_tracker_request(metainfo, rx_buf).await?;
        // Here you would typically parse the tracker's response and transition to the next state
        // For this example, we'll just log the raw response
        let tracker_response = TrackerResponse::parse(&rx_buf[..bytes_written])
            .map_err(|_| BitTorrenterError::TrackerResponseParseError)?;

        defmt_or_log::info!("Received tracker response: {:?}", tracker_response);

        Ok(BitTorrenter {
            net: self.net,
            fs: self.fs,
            socket_buffers: self.socket_buffers,
            peer_id: self.peer_id,
            port: self.port,
            state: Downloading::new(tracker_response.peers, metainfo),
        })
    }
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
            .net
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
