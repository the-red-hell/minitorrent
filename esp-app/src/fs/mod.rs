use core_logic::fs::FileSystem;
use esp_hal::{gpio, spi};

use crate::fs::sd_card::{SPIInitializer, SdCardError};
use volume_mgr::EspVolumeMgr;

pub mod sd_card;
pub mod volume_mgr;

/// Initializes the SD Card.
/// Creates a Volume Manager with this SD Card.
/// You can optain the FileSystem by calling `FileSystem::new()`.
pub async fn initialize_esp_fs<SPI, SCK, MISO, MOSI, CS>(
    initializer: SPIInitializer<SCK, MISO, MOSI, CS>,
    spi: SPI,
) -> Result<FileSystem<EspVolumeMgr>, SdCardError>
where
    SPI: spi::master::Instance + 'static,
    SCK: gpio::interconnect::PeripheralOutput<'static>,
    MISO: gpio::interconnect::PeripheralInput<'static>,
    MOSI: gpio::interconnect::PeripheralOutput<'static>,
    CS: gpio::OutputPin + 'static,
{
    let sd_card = sd_card::SdCard::init(initializer, spi)?;

    let volume_mgr = sd_card.into_volume_mgr();

    Ok(FileSystem::new(volume_mgr))
}
