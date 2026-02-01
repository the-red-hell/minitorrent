use embedded_sdmmc::{BlockDevice, RawDirectory, RawFile, filesystem::ToShortFileName};

use crate::fs::{FileSystem, FileSystemExt, VolumeMgr};

impl<V> FileSystem<V>
where
    V: VolumeMgr,
{
    pub fn new(volume_mgr: V) -> Self {
        let vol0 = volume_mgr.get_vol0();
        let root_dir = volume_mgr.get_root_dir(vol0);

        Self {
            volume_mgr,
            vol0,
            opened_dir: root_dir,
            open_file: None,
        }
    }

    pub fn get_volume_mgr(&self) -> &V {
        &self.volume_mgr
    }

    pub fn get_current_dir(&self) -> RawDirectory {
        self.opened_dir
    }

    pub fn go_to_root_dir(&mut self) {
        self.swap_current_dir(self.volume_mgr.get_root_dir(self.vol0));
    }

    fn swap_current_dir(&mut self, dir: RawDirectory) {
        let dir = core::mem::replace(&mut self.opened_dir, dir);
        self.get_volume_mgr()
            .close_dir(dir)
            .expect("Should not fail to close dir");
    }

    pub fn get_open_file(&self) -> Option<RawFile> {
        self.open_file
    }

    /// Set the open file and return the previous one.
    fn close_open_file(&mut self) {
        if let Some(file) = self.open_file {
            self.get_volume_mgr()
                .close_file(file)
                .expect("Should not fail to close file");
        }
    }
}

impl<V> FileSystemExt for FileSystem<V>
where
    V: VolumeMgr,
{
    type Error = embedded_sdmmc::Error<<V::BlockDevice as BlockDevice>::Error>;

    async fn write_to_opened_file(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.get_volume_mgr().write(
            self.get_open_file()
                .ok_or(embedded_sdmmc::Error::BadHandle)?,
            buf,
        )
    }

    async fn read_to_end(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.get_volume_mgr().read(
            self.get_open_file()
                .ok_or(embedded_sdmmc::Error::BadHandle)?,
            buf,
        )
    }

    fn open_file<N: ToShortFileName>(
        &mut self,
        file_name: N,
        mode: embedded_sdmmc::Mode,
    ) -> Result<(), Self::Error> {
        self.close_open_file();

        let raw_file = self
            .volume_mgr
            .open_file_in_dir(self.get_current_dir(), file_name, mode)?;

        self.open_file = Some(raw_file);

        Ok(())
    }

    fn open_dir<N: ToShortFileName>(&mut self, dir_name: N) -> Result<(), Self::Error> {
        let raw_dir = self
            .get_volume_mgr()
            .open_dir(self.get_current_dir(), dir_name)?;

        self.swap_current_dir(raw_dir);

        Ok(())
    }
}
