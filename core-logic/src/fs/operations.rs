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
            opened_dir: Some(root_dir),
            open_file: None,
        }
    }

    pub fn get_volume_mgr(&self) -> &V {
        &self.volume_mgr
    }

    pub fn take_current_dir(&mut self) -> Option<RawDirectory> {
        self.opened_dir.take()
    }

    pub fn go_to_root_dir(&mut self) {
        self.set_current_dir(self.volume_mgr.get_root_dir(self.vol0));
    }

    fn set_current_dir(&mut self, dir: RawDirectory) {
        let dir = core::mem::replace(&mut self.opened_dir, Some(dir));
        if let Some(dir) = dir {
            let _closing_result = self.get_volume_mgr().close_dir(dir);
        }
    }

    pub fn take_open_file(&mut self) -> Option<RawFile> {
        self.open_file.take()
    }

    /// Set the open file and return the previous one.
    fn _set_open_file(&mut self, file: RawFile) {
        if let Some(file) = self.open_file.replace(file) {
            let _closing_result = self.get_volume_mgr().close_file(file);
        }
    }
}

impl<V> FileSystemExt for FileSystem<V>
where
    V: VolumeMgr,
{
    type Error = embedded_sdmmc::Error<<V::BlockDevice as BlockDevice>::Error>;

    fn _write_to_opened_file(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        let file = self
            .take_open_file()
            .expect("File not opened!")
            .to_file(self.get_volume_mgr());

        file.write(buf)
    }

    fn _read_to_end(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let file = self
            .take_open_file()
            .expect("File not opened!")
            .to_file(self.get_volume_mgr());

        file.read(buf)
    }

    fn _open_file<N: ToShortFileName>(&mut self, file_name: N) -> Result<(), Self::Error> {
        let raw_file = {
            let dir = if let Some(dir) = self.take_current_dir() {
                dir.to_directory(self.get_volume_mgr())
            } else {
                return Err(embedded_sdmmc::Error::BadHandle);
            };
            dir.open_file_in_dir(file_name, embedded_sdmmc::Mode::ReadWriteCreateOrAppend)?
                .to_raw_file()
        };
        self._set_open_file(raw_file);

        Ok(())
    }

    fn open_dir<N: ToShortFileName>(&mut self, dir_name: N) -> Result<(), Self::Error> {
        let raw_dir = {
            let dir = if let Some(dir) = self.take_current_dir() {
                dir.to_directory(self.get_volume_mgr())
            } else {
                return Err(embedded_sdmmc::Error::BadHandle);
            };
            dir.open_dir(dir_name)?.to_raw_directory()
        };
        self.set_current_dir(raw_dir);

        Ok(())
    }
}
