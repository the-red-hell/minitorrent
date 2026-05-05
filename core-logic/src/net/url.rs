//! A simple URL parser for embedded systems without atomic operations (`url` uses `fetch_add`)
use heapless::String;

#[derive(Clone)]
#[defmt_or_log::derive_format_or_debug]
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
        if scheme == "https" {
            return Err("HTTPS is not supported in this simple URL parser");
        }

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
    #[inline]
    pub const fn host_str(&self) -> Option<&str> {
        Some(self.host)
    }

    /// Get the port, or default based on scheme
    #[inline]
    pub fn port(&self) -> Option<u16> {
        self.port.or(match self.scheme {
            "http" => Some(80),
            "https" => Some(443),
            _ => None,
        })
    }

    /// Get the path
    #[inline]
    pub const fn path(&self) -> &str {
        self.path
    }

    /// Get the query string
    #[inline]
    pub fn query(&self) -> Option<&str> {
        self.query.as_ref().map(|s| s.as_str())
    }

    /// Set the query string
    #[inline]
    pub fn set_query(&mut self, query: Option<&str>) {
        self.query = query.map(|q| {
            let mut s = String::<512>::new();
            s.push_str(q).ok();
            s
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
