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

    pub fn get_current_dir(&self) -> &RawDirectory {
        &self.opened_dir
    }

    pub fn go_to_root_dir(&mut self) {
        self.set_current_dir(self.volume_mgr.get_root_dir(self.vol0));
    }

    fn set_current_dir(&mut self, dir: RawDirectory) {
        let dir = core::mem::replace(&mut self.opened_dir, dir);
        self.get_volume_mgr()
            .close_dir(dir)
            .expect("Directory could not be closed.");
    }

    pub fn get_open_file(&self) -> Option<&RawFile> {
        self.open_file.as_ref()
    }

    /// Set the open file and return the previous one.
    fn set_open_file(&mut self, file: RawFile) {
        if let Some(file) = self.open_file.replace(file) {
            self.get_volume_mgr()
                .close_file(file)
                .expect("File could not be closed.");
        }
    }
}

impl<V> FileSystemExt for FileSystem<V>
where
    V: VolumeMgr,
{
    type Error = embedded_sdmmc::Error<<V::B as BlockDevice>::Error>;

    fn _write_to_opened_file(&self, buf: &[u8]) -> Result<(), Self::Error> {
        let file = self
            .get_open_file()
            .expect("File not opened!")
            .to_file(self.get_volume_mgr());

        file.write(buf)
    }

    fn read_to_end(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let file = self
            .get_open_file()
            .expect("File not opened!")
            .to_file(self.get_volume_mgr());

        file.read(buf)
    }

    fn open_file<N: ToShortFileName>(&mut self, file_name: N) -> Result<(), Self::Error> {
        let raw_file = {
            let dir = self.get_current_dir().to_directory(self.get_volume_mgr());
            dir.open_file_in_dir(file_name, embedded_sdmmc::Mode::ReadWriteCreateOrAppend)?
                .to_raw_file()
        };
        self.set_open_file(raw_file);

        Ok(())
    }

    fn open_dir<N: ToShortFileName>(&mut self, dir_name: N) -> Result<(), Self::Error> {
        let raw_dir = {
            let dir = self.get_current_dir().to_directory(self.get_volume_mgr());
            dir.open_dir(dir_name)?.to_raw_directory()
        };
        self.set_current_dir(raw_dir);

        Ok(())
    }
}
