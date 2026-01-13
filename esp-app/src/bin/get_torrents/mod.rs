use core::cell::OnceCell;

use alloc::{string::ToString as _, vec, vec::Vec};
use embedded_sdmmc::LfnBuffer;
use minitorrent::fs::FileSystem;

/// Get's the first torrent file in the 'torrents' directory.
/// Make sure to put the torrent file in the 'torrents' directory as well as have the directory in the root of the filesystem.
pub(super) async fn get_torrent() -> Option<Vec<u8>> {
    let fs = FileSystem::new();
    fs.with_volume(|v| {
        let root_dir = v.open_root_dir().expect("Root dir not found.");
        let torrents = root_dir
            .open_dir("torrents")
            .expect("'torrents' directory not found.");

        let mut lfn_buffer_storage = [0; 20];
        let mut lfn_buffer = LfnBuffer::new(&mut lfn_buffer_storage);
        let file_name = OnceCell::new();
        torrents
            .iterate_dir_lfn(&mut lfn_buffer, |dir, name| {
                if let Some(name) = name
                    && name.ends_with("torrent")
                    && file_name.get().is_none()
                {
                    defmt::trace!("found torrent: {}", name);
                    file_name
                        .set(dir.name.clone())
                        .expect("we checked that it is uninitialized");
                } else {
                    defmt::trace!("found file to ignore: {}", name);
                }
            })
            .expect("Couldn't iterate dir");

        if let Some(file_name) = file_name.get() {
            let file = torrents
                .open_file_in_dir(file_name, embedded_sdmmc::Mode::ReadOnly)
                .expect("File not found");
            let mut buf = vec![0u8; file.length() as usize];

            file.read(&mut buf).expect("Couldn't read file");
            defmt::info!("Using torrent-file {}", file_name.to_string().as_str());
            Some(buf)
        } else {
            None
        }
    })
    .await
}
