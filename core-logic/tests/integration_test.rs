use crate::bittorrenter_helper::init_bittorrenter;
use core_logic::{core::metainfo::MetaInfoFile, fs::FileSystemExt};

mod bittorrenter_helper;
mod fs_helper;
mod wifi_helper;

#[tokio::test]
#[ignore = "Won't work on GitHub actions. Run with `cargo test -- --ignored` to execute."]
async fn integration_test() {
    env_logger::init();
    let mut bittorrenter = init_bittorrenter();
    let mut buf = [0u8; 1024 * 10];
    let file_length = bittorrenter
        .fs()
        .put_torrent_into_buf(&mut buf)
        .await
        .unwrap();
    let metadata = MetaInfoFile::parse(&buf[..file_length]).unwrap();

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
    let mut downloader = bittorrenter
        .into_downloader(&metadata, &mut rx_buf)
        .await
        .unwrap();

    downloader.download().await.unwrap();

    downloader.fs().go_to_root_dir();
    downloader
        .fs()
        .open_file("sample.txt", embedded_sdmmc::Mode::ReadOnly)
        .unwrap();
    let mut buf = vec![0u8; 92063];
    downloader.fs().read_to_end(&mut buf).await.unwrap();
    assert!(buf.starts_with(b"## What Is a Hacker?"));
    assert!(buf.ends_with(b"that it could be inside `HourlyEmployee`."));
}
