use embedded_sdmmc::{Directory, Error, ShortFileName};

use crate::fs_helper::{
    TORRENT_STRING,
    blockdevice::{Clock, LinuxBlockDevice},
    init_fs_duple,
};

mod fs_helper;

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

/// Recursively print a directory listing for the open directory given.
///
/// The path is for display purposes only.
///
/// props to: https://github.com/rust-embedded-community/embedded-sdmmc-rs/blob/8d30ebcf7d3753d7f3f984a43934e69fa9d589d9/examples/list_dir.rs
pub fn list_dir(
    directory: Directory<'_, LinuxBlockDevice, Clock, 4, 4, 1>,
    path: &str,
) -> Result<(), Error<<LinuxBlockDevice as embedded_sdmmc::BlockDevice>::Error>> {
    println!("Listing {}", path);
    let mut children = Vec::new();
    directory.iterate_dir(|entry| {
        println!(
            "{:12} {:9} {} {}",
            entry.name,
            entry.size,
            entry.mtime,
            if entry.attributes.is_directory() {
                "<DIR>"
            } else {
                ""
            }
        );
        if entry.attributes.is_directory()
            && entry.name != ShortFileName::parent_dir()
            && entry.name != ShortFileName::this_dir()
        {
            children.push(entry.name.clone());
        }
    })?;
    for child_name in children {
        let child_dir = directory.open_dir(&child_name)?;
        let child_path = if path == "/" {
            format!("/{}", child_name)
        } else {
            format!("{}/{}", path, child_name)
        };
        list_dir(child_dir, &child_path)?;
    }
    Ok(())
}
