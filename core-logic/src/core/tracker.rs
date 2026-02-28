use crate::core::{InfoHash, PeerId, net::percent_encode};
use alloc::string::String;
use core::fmt::Write;
use heapless::Vec;

#[derive(Debug, Clone)]
pub struct TrackerRequest<'a> {
    /// the info hash of the torrent
    pub info_hash: &'a InfoHash,
    /// a unique identifier for your client
    peer_id: &'a PeerId,
    /// the port your client is listening on
    port: u16,
    /// the total amount uploaded so far
    uploaded: u32,
    /// the total amount downloaded so far
    downloaded: u32,
    /// the number of bytes left to download
    left: u32,
    /// whether the peer list should use the compact representation
    /// The compact representation is more commonly used in the wild, the non-compact representation is mostly supported for backward-compatibility.
    compact: u8,
}

impl<'a> TrackerRequest<'a> {
    pub fn new(info_hash: &'a InfoHash, peer_id: &'a PeerId, port: u16, left: u32) -> Self {
        Self {
            info_hash,
            peer_id,
            port,
            uploaded: 0,
            downloaded: 0,
            left,
            compact: 1,
        }
    }

    pub(crate) fn to_url_encoded(&self) -> String {
        let mut url_encoded = String::with_capacity(256);

        write!(url_encoded, "info_hash={}", &percent_encode(self.info_hash)).unwrap();
        write!(url_encoded, "&peer_id={}", &percent_encode(self.peer_id)).unwrap();
        write!(url_encoded, "&port={}", self.port).unwrap();
        write!(url_encoded, "&uploaded={}", self.uploaded).unwrap();
        write!(url_encoded, "&downloaded={}", self.downloaded).unwrap();
        write!(url_encoded, "&left={}", self.left).unwrap();
        write!(url_encoded, "&compact={}", self.compact).unwrap();
        url_encoded
    }
}

#[derive(Debug)]
pub struct TrackerResponse {
    pub interval: u32,
    pub peers: Vec<core::net::SocketAddrV4, 10>,
}

impl defmt::Format for TrackerResponse {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "TrackerResponse {{ interval: {}, peers: {:?} }}",
            self.interval,
            defmt::Debug2Format(&self.peers)
        );
    }
}

mod tracker_response_parser {
    use bencode::{BencodeParser, Result};
    use heapless::Vec;

    use crate::core::tracker::TrackerResponse;

    impl TrackerResponse {
        pub fn parse(input: &[u8]) -> Result<Self> {
            let mut p = BencodeParser::new(input);

            let mut interval = 0;
            let mut peers: Vec<core::net::SocketAddrV4, 10> = Vec::new();

            p.expect_dict_start()?;

            while !p.match_dict_end() {
                let key = p.parse_str()?;

                match key {
                    "interval" => {
                        interval = p.parse_int()? as u32;
                    }
                    "peers" => {
                        let peer_bytes = p.parse_raw_value()?;
                        // Compact peer list parsing
                        let peer_chunks = peer_bytes.as_chunks::<6>();

                        peers.extend(peer_chunks.0.iter().take(peers.capacity()).map(|chunk| {
                            let ip =
                                core::net::Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                            core::net::SocketAddrV4::new(ip, port)
                        }));
                    }
                    _ => {
                        p.skip_any()?;
                    }
                }
            }

            Ok(TrackerResponse { interval, peers })
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tracker_request_url_encoding() {
        let info_hash: InfoHash = [0u8; 20];
        let peer_id: PeerId = [1u8; 20];
        let request = TrackerRequest::new(&info_hash, &peer_id, 6881, 1000);

        let url_encoded = request.to_url_encoded();
        assert!(
            url_encoded
                .contains("info_hash=%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00")
        );
        assert!(
            url_encoded
                .contains("peer_id=%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01")
        );
        assert!(url_encoded.contains("port=6881"));
        assert!(url_encoded.contains("uploaded=0"));
        assert!(url_encoded.contains("downloaded=0"));
        assert!(url_encoded.contains("left=1000"));
        assert!(url_encoded.contains("compact=1"));
    }
}
