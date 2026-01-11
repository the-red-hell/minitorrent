use alloc::{string::ToString, vec, vec::Vec};
use minitorrent::fs::FileSystem;

pub(super) async fn get_torrent() -> Vec<u8> {
    let fs = FileSystem::new();
    fs.with_volume(|v| {
        let root_dir = v.open_root_dir().expect("Root dir not found.");
        let torrents = root_dir
            .open_dir("torrents")
            .expect("'torrents' directory not found.");

        torrents
            .iterate_dir(|dir| {
                let name = dir.name.to_string();
                defmt::info!("{}", name.as_str());
            })
            .unwrap();

        let file = torrents
            .open_file_in_dir("other.tor", embedded_sdmmc::Mode::ReadOnly)
            .expect("File not found");
        let mut buf = vec![0u8; file.length() as usize];

        file.read(&mut buf).expect("Couldn't read file");
        buf
    })
    .await
}
