use embedded_sdmmc::ShortFileName;

use crate::fs_duples::{init_fs_duple, list_dir};

mod fs_duples;

#[tokio::test]
async fn test_retrieve_torrent() {
    let mut fs_duple = init_fs_duple();
    let torrent = fs_duple.get_torrent_from_file().await;

    dbg!(torrent);
}

#[test]
fn list_directories() {
    let fs_duple = init_fs_duple();

    let root_dir = fs_duple
        .get_current_dir() // always root dir at init
        .to_directory(fs_duple.get_volume_mgr());
    list_dir(root_dir, "/").unwrap();
}
