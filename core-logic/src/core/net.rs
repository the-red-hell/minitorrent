use alloc::string::String;
use core::fmt::Write;

pub fn percent_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(60);
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
}
