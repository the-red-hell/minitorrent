use embedded_sdmmc::{RawDirectory, RawFile, RawVolume, filesystem::ToShortFileName};

mod operations;
pub mod torrent_retrieval;
mod volume_mgr;
pub use volume_mgr::VolumeMgr;

pub(crate) trait FileSystemExt {
    type Error: core::fmt::Debug;

    /// opens a file with ReadWriteCreateOrAppend mode
    fn open_file<N: ToShortFileName>(&mut self, file_name: N) -> Result<(), Self::Error>;

    fn open_dir<N: ToShortFileName>(&mut self, dir_name: N) -> Result<(), Self::Error>;

    fn _write_to_opened_file(&self, buf: &[u8]) -> Result<(), Self::Error>;

    fn read_to_end(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Struct to interact with the filesystem on the ESP32C3.
pub struct FileSystem<V>
where
    V: VolumeMgr,
{
    volume_mgr: V,
    vol0: RawVolume,
    /// The directory that is currently open.
    /// At the beginning this will be the root directory of the filesystem.
    opened_dir: RawDirectory,
    // TODO: allow multiple opened files (two, for DB and file which is written to)
    /// The file that is currently open.
    open_file: Option<RawFile>,
}

impl<V> Drop for FileSystem<V>
where
    V: VolumeMgr,
{
    fn drop(&mut self) {
        // Close directory
        self.get_volume_mgr()
            .close_dir(self.opened_dir)
            .expect("Directory could not be closed.");

        // Close file
        if let Some(file) = self.open_file {
            self.get_volume_mgr()
                .close_file(file)
                .expect("File could not be closed.");
        }

        // Close volume
        self.get_volume_mgr()
            .close_volume(self.vol0)
            .expect("Volume could not be closed.");
    }
}
