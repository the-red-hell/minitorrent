use core_logic::fs::FileSystem;
use embedded_sdmmc::{RawDirectory, RawFile, RawVolume};
use esp_hal::{gpio, spi};

use crate::fs::sd_card::{
    SPIInitializer, SdCardError,
    volume_mgr::{VolumeMgr, VolumeMgrType},
};

pub mod sd_card;

/// Struct to interact with the filesystem on the ESP32C3.
pub struct EspFileSystem {
    // TODO: allow multiple opened files (two, for DB and file which is written to)
    volume_mgr: VolumeMgr,
    vol0: RawVolume,
    /// The directory that is currently open.
    /// At the beginning this will be the root directory of the filesystem.
    opened_dir: RawDirectory,
    /// The file that is currently open.
    open_file: Option<RawFile>,
}

impl EspFileSystem {
    /// Initializes the SD Card.
    /// Creates a Volume Manager with this SD Card.
    /// You can optain the FileSystem by calling `FileSystem::new()`.
    pub async fn setup<SPI, SCK, MISO, MOSI, CS>(
        initializer: SPIInitializer<SCK, MISO, MOSI, CS>,
        spi: SPI,
    ) -> Result<Self, SdCardError>
    where
        SPI: spi::master::Instance + 'static,
        SCK: gpio::interconnect::PeripheralOutput<'static>,
        MISO: gpio::interconnect::PeripheralInput<'static>,
        MOSI: gpio::interconnect::PeripheralOutput<'static>,
        CS: gpio::OutputPin + 'static,
    {
        let sd_card = sd_card::SdCard::init(initializer, spi)?;

        let volume_mgr = sd_card.into_volume_mgr();
        let vol0 = volume_mgr.get_vol0();
        let root_dir = volume_mgr.get_root_dir(vol0);

        let esp_file_system = Self {
            volume_mgr,
            vol0,
            opened_dir: root_dir,
            open_file: None,
        };

        Ok(esp_file_system)
    }

    pub fn get_volume_mgr(&self) -> &VolumeMgrType<'static> {
        &self.volume_mgr.0
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

impl FileSystem for EspFileSystem {
    type Error = embedded_sdmmc::Error<embedded_sdmmc::SdCardError>;

    async fn write_to_opened_file(&self, buf: &[u8]) -> Result<(), Self::Error> {
        let file = self
            .get_open_file()
            .expect("File not opened!")
            .to_file(self.get_volume_mgr());

        file.write(buf)
    }

    async fn read_to_end(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let file = self
            .get_open_file()
            .expect("File not opened!")
            .to_file(self.get_volume_mgr());

        file.read(buf)
    }

    fn open_file(&mut self, file_name: &str) -> Result<(), Self::Error> {
        let raw_file = {
            let dir = self.get_current_dir().to_directory(self.get_volume_mgr());
            dir.open_file_in_dir(file_name, embedded_sdmmc::Mode::ReadWriteCreateOrAppend)?
                .to_raw_file()
        };
        self.set_open_file(raw_file);

        Ok(())
    }

    fn open_dir(&mut self, dir_name: &str) -> Result<(), Self::Error> {
        let raw_dir = {
            let dir = self.get_current_dir().to_directory(self.get_volume_mgr());
            dir.open_dir(dir_name)?.to_raw_directory()
        };
        self.set_current_dir(raw_dir);

        Ok(())
    }
}

impl Drop for EspFileSystem {
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
