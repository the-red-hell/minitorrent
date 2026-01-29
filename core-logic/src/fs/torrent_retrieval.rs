use alloc::{string::ToString as _, vec, vec::Vec};
use embedded_sdmmc::LfnBuffer;

use crate::fs::{FileSystem, FileSystemExt, VolumeMgr};

impl<V> FileSystem<V>
where
    V: VolumeMgr,
{
    /// Get's the first torrent file in the 'torrents' directory.
    /// Make sure to put the torrent file in the 'torrents' directory as well as have the directory in the root of the filesystem.
    pub async fn get_torrent_from_file(&mut self) -> Option<Vec<u8>> {
        self.go_to_root_dir();
        self.open_dir("torrents")
            .expect("'torrents' directory not found.");
        let torrents = self
            .take_current_dir()
            .expect("root dir should be open")
            .to_directory(self.get_volume_mgr());

        let mut lfn_buffer_storage = [0; 20];
        let mut lfn_buffer = LfnBuffer::new(&mut lfn_buffer_storage);
        let mut file_name = None;
        torrents
            .iterate_dir_lfn(&mut lfn_buffer, |dir, name| {
                if let Some(name) = name
                    && name.ends_with("torrent")
                    && file_name.is_none()
                {
                    defmt::trace!("found torrent: {}", name);
                    file_name = Some(dir.name.clone());
                } else {
                    defmt::trace!("found file to ignore: {}", name);
                }
            })
            .expect("Couldn't iterate dir");

        if let Some(file_name) = file_name.as_ref() {
            let file = torrents
                .open_file_in_dir(file_name, embedded_sdmmc::Mode::ReadOnly)
                .expect("we just found the file with this name");
            let mut buf = vec![0u8; file.length() as usize];

            file.read(&mut buf).expect("Couldn't read file");
            defmt::info!("Using torrent-file {}", file_name.to_string().as_str());
            Some(buf)
        } else {
            None
        }
    }
}
