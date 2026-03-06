use ::core::fmt::Write as _;
use heapless::String;

pub(crate) mod buffer;
mod downloader;
pub mod tcp;
mod tracker_requester;
mod url;

pub(crate) fn percent_encode(bytes: &[u8]) -> String<60> {
    let mut encoded = String::<60>::new();
    for &b in bytes {
        write!(encoded, "%{:02X}", b).unwrap();
    }
    encoded
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{InfoHash, PeerId},
        net::percent_encode,
    };

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
