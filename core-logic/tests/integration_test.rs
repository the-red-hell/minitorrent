use crate::bittorrenter_helper::init_bittorrenter;
use core_logic::core::{metainfo::MetaInfoFile, tracker::TrackerResponse};

mod bittorrenter_helper;
mod fs_helper;
mod wifi_helper;

#[tokio::test]
async fn integration_test() {
    let mut bittorrenter = init_bittorrenter();
    let torrent = bittorrenter.fs().get_torrent_from_file().await.unwrap();
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

    let mut rx_buf = vec![0u8; 1024 * 10];
    let response = bittorrenter
        .make_tracker_request(&metadata, &mut rx_buf)
        .await
        .unwrap();

    assert!(response > 0);

    // print tracker response as string for debugging
    let response_str = String::from_utf8_lossy(&rx_buf[..response]);
    println!("Tracker response:\n{}", response_str);

    let tracker_response = TrackerResponse::parse(&rx_buf[..response]).unwrap();
    assert_eq!(tracker_response.peers.len(), 3);
    dbg!(tracker_response);

    // Further processing of the response can be done here
}
