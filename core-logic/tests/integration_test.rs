use core_logic::{core::metainfo::MetaInfoFile, wifi::WifiStack};

use crate::fs_helper::init_fs_duple;

mod fs_helper;
mod wifi_helper;

#[tokio::test]
async fn integration_test() {
    let mut fs_duple = init_fs_duple();
    let torrent = fs_duple.get_torrent_from_file().await.unwrap();
    let metadata = MetaInfoFile::parse(&torrent).unwrap();

    assert_eq!(
        metadata.announce,
        "http://bittorrent-test-tracker.codecrafters.io/announce"
    );
    assert_eq!(metadata.info.length, 92063);
    assert_eq!(
        hex::encode(metadata.info_hash),
        "d69f91e6b2ae4c542468d1073a71d4ea13879a7f"
    );

    let wifi_stack = wifi_helper::WifiStackDuple;
    let mut rx_buf = vec![0u8; 1024 * 10];
    let response = wifi_stack
        .make_http_request(metadata.announce, &mut rx_buf)
        .await
        .unwrap();

    assert!(response.len() > 0);

    // Further processing of the response can be done here
}
