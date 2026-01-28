use embedded_sdmmc::ShortFileName;

use crate::fs_duples::init_fs_duple;

mod fs_duples;

#[tokio::test]
async fn test_retrieve_torrent() {
    let mut fs_duple = init_fs_duple();

    println!("Listing {}", "/");
    let mut children = Vec::new();
    let directory = fs_duple
        .get_current_dir()
        .to_directory(fs_duple.get_volume_mgr());
    directory
        .iterate_dir(|entry| {
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
        })
        .unwrap();
    for child_name in children {
        let child_dir = directory.open_dir(&child_name).unwrap();
        let child_path = format!("/{}", child_name);
        // list_dir(child_dir, &child_path).unwrap();
    }
    // let torrent = fs_duple.get_torrent_from_file().await;

    // dbg!(torrent);
}
