use crate::fs_duples::{init_fs_duple, list_dir};

mod fs_duples;

pub const TORRENT_STRING: &'static [u8] = include_bytes!("sample.torrent");

#[tokio::test]
async fn test_retrieve_torrent() {
    let mut fs_duple = init_fs_duple();
    let torrent = fs_duple.get_torrent_from_file().await;

    assert_eq!(torrent.unwrap().as_slice(), TORRENT_STRING);
}

#[test]
fn list_directories() {
    let mut fs_duple = init_fs_duple();

    let root_dir = fs_duple
        .take_current_dir()
        .expect("always root dir at init")
        .to_directory(fs_duple.get_volume_mgr());
    list_dir(root_dir, "/").unwrap();
}
