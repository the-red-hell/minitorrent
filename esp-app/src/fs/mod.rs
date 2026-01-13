use embassy_sync::once_lock::OnceLock;
use esp_hal::{gpio, spi};

use crate::fs::sd_card::{
    FileSystemInitializer, SdCardError,
    volume::{Volume, VolumeMgr},
};

pub mod sd_card;

static VOLUME_MGR: OnceLock<VolumeMgr> = OnceLock::new();

/// Struct to
pub struct FileSystem;

impl FileSystem {
    pub async fn initialize<SPI, SCK, MISO, MOSI, CS>(
        initializer: FileSystemInitializer<'static, SCK, MISO, MOSI, CS>,
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

        VOLUME_MGR
            .init(sd_card.to_volume_mgr())
            .map_err(|_| ())
            .expect("Shall not be initialized yet");

        Ok(FileSystem)
    }

    /// Calling this function creates a new instance of the FileSystem.
    /// If the FileSystem has not been initialized via `initialize`, all operations on it will block and even busy-spin.
    /// So make sure you initialize it before using it.
    pub fn new() -> Self {
        Self
    }

    /// This helper method handles the Critical Section boilerplate.
    /// It gives the user temporary access to the Volume.
    pub async fn with_volume<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Volume) -> R,
    {
        let mgr = VOLUME_MGR.get().await;

        let volume0 = mgr.get_volume();
        f(&volume0)
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}
