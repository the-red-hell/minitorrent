use embedded_sdmmc::{RawDirectory, RawFile, RawVolume, filesystem::ToShortFileName};

mod operations;
pub mod torrent_retrieval;
mod volume_mgr;
pub use volume_mgr::VolumeMgr;

/// A trait that provides some common operations for the filesystem.
/// This is mainly for the abstraction between my own filesystem implementation and `embedded_sdmmc`
/// (my code - `FileSystemExt`` - embedded_sdmmc)
/// This is still depended on embedded_sdmmc for the open-mode, but making an enum for it shouldn't be too hard.
/// I won't do this as as long as there's no async embedded-sdmmc implementation.
#[allow(async_fn_in_trait)]
pub trait FileSystemExt {
    type Error: core::fmt::Debug;

    /// opens a file with ReadWriteCreateOrAppend mode
    fn open_file<N: ToShortFileName>(
        &mut self,
        file_name: N,
        mode: embedded_sdmmc::Mode,
    ) -> Result<(), Self::Error>;

    fn open_dir<N: ToShortFileName>(&mut self, dir_name: N) -> Result<(), Self::Error>;

    async fn write_to_opened_file(&mut self, buf: &[u8]) -> Result<(), Self::Error>;

    async fn read_to_end(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Struct to provide an abstraction over the filesystem.
/// It can do basic operations like opening files and directories,
/// but if you want more, you'll probably have to use the `embedded_sdmmc::VolumeManager::...` methods.
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
        let dir = self.get_current_dir();
        let _close_dir_result = self.get_volume_mgr().close_dir(dir);

        // Close file
        if let Some(file) = self.open_file {
            let _close_file_result = self.get_volume_mgr().close_file(file);
        }

        // Close volume
        self.get_volume_mgr()
            .close_volume(self.vol0)
            .expect("Volume could not be closed.");
    }
}
