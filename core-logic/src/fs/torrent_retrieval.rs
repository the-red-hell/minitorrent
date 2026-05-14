use alloc::string::ToString as _;
use embedded_sdmmc::LfnBuffer;

use crate::fs::{FileSystem, FileSystemExt, VolumeMgr};

impl<V> FileSystem<V>
where
    V: VolumeMgr,
{
    /// Get's the first torrent file in the 'torrents' directory.
    /// Make sure to put the torrent file in the 'torrents' directory as well as have the directory in the root of the filesystem.
    /// Returns the length of the torrent file.
    pub async fn put_torrent_into_buf(&mut self, buf: &mut [u8]) -> Option<usize> {
        self.go_to_root_dir();
        self.open_dir("torrents")
            .expect("'torrents' directory not found.");

        let mut lfn_buffer_storage = [0; 20];
        let mut lfn_buffer = LfnBuffer::new(&mut lfn_buffer_storage);
        let mut file_name = None;
        self.get_volume_mgr()
            .iterate_dir_lfn(self.get_current_dir(), &mut lfn_buffer, |dir, name| {
                if let Some(name) = name
                    && name.ends_with("torrent")
                    && file_name.is_none()
                {
                    defmt_or_log::trace!("found torrent: {}", name);
                    file_name = Some(dir.name.clone());
                } else {
                    defmt_or_log::trace!("found file to ignore: {:?}", name);
                }
            })
            .expect("Couldn't iterate dir");

        if let Some(file_name) = file_name.as_ref() {
            self.open_file(file_name, embedded_sdmmc::Mode::ReadOnly)
                .expect("we just found the file with this name");

            let file_length = self
                .volume_mgr
                .file_length(self.get_open_file().expect("we just opened it"))
                .unwrap() as usize;
            if file_length > buf.len() {
                defmt_or_log::error!("Torrent file is too big. Max size is {}", buf.len());
                return None;
            }

            self.read_to_end(buf).await.expect("Couldn't read file");
            defmt_or_log::info!("Using torrent-file {}", file_name.to_string().as_str());
            Some(file_length)
        } else {
            None
        }
    }
}
