pub mod volume_mgr;

use core::marker::PhantomData;

use defmt::{info, warn};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{TimeSource, sdcard::AcquireOpts};
use esp_hal::{
    Blocking,
    delay::Delay,
    gpio,
    spi::{self, master::Spi},
    time,
};

use crate::fs::sd_card::volume_mgr::VolumeMgr;

#[derive(Debug)]
pub enum SdCardError {
    SPIError,
}

pub(super) struct SdCard(
    embedded_sdmmc::SdCard<
        ExclusiveDevice<Spi<'static, Blocking>, gpio::Output<'static>, Delay>,
        Delay,
    >,
);

/// A struct holding all the data of the physically connected pins to the SD Card module.
pub struct SPIInitializer<SCK, MISO, MOSI, CS>
where
    SCK: gpio::interconnect::PeripheralOutput<'static>,
    MISO: gpio::interconnect::PeripheralInput<'static>,
    MOSI: gpio::interconnect::PeripheralOutput<'static>,
    CS: gpio::OutputPin,
{
    sck: SCK,
    miso: MISO,
    mosi: MOSI,
    cs: CS,
    _phantom: PhantomData<&'static bool>,
}
impl<SCK, MISO, MOSI, CS> SPIInitializer<SCK, MISO, MOSI, CS>
where
    SCK: gpio::interconnect::PeripheralOutput<'static>,
    MISO: gpio::interconnect::PeripheralInput<'static>,
    MOSI: gpio::interconnect::PeripheralOutput<'static>,
    CS: gpio::OutputPin,
{
    pub fn new(sck: SCK, miso: MISO, mosi: MOSI, cs: CS) -> Self {
        SPIInitializer {
            sck,
            miso,
            mosi,
            cs,
            _phantom: Default::default(),
        }
    }
}

impl SdCard {
    /// Initializes the SD Card.
    pub(super) fn init<SPI, SCK, MISO, MOSI, CS>(
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
        let spi_bus = Spi::new(
            spi,
            spi::master::Config::default().with_frequency(time::Rate::from_khz(100)),
        )
        .map_err(|_| SdCardError::SPIError)?
        .with_sck(initializer.sck)
        .with_miso(initializer.miso)
        .with_mosi(initializer.mosi);

        let cs = gpio::Output::new(
            initializer.cs,
            esp_hal::gpio::Level::High,
            gpio::OutputConfig::default(),
        );

        let delay = Delay::new();

        let spi_device =
            ExclusiveDevice::new(spi_bus, cs, delay).map_err(|_| SdCardError::SPIError)?;

        // --- SD Card ---
        let sdcard = embedded_sdmmc::SdCard::new_with_options(
            spi_device,
            delay,
            AcquireOpts {
                use_crc: false,
                acquire_retries: 50,
            },
        );
        // println!("Card size is {:?} MB", sdcard.num_bytes());
        let num_bytes = loop {
            match sdcard.num_bytes() {
                Ok(num_bytes) => break num_bytes,
                Err(e) => {
                    warn!("failed to initialize card with error {:?}", e);
                    delay.delay_millis(500);
                }
            }
        };
        info!("Card size is {:?} MB", num_bytes);

        sdcard.spi(|spi| {
            spi.bus_mut()
                .apply_config(
                    &spi::master::Config::default().with_frequency(time::Rate::from_mhz(25)),
                )
                .expect("Couldn't set to high-frequency.");
        });

        Ok(SdCard(sdcard))
    }

    pub(super) fn into_volume_mgr(self) -> VolumeMgr {
        self.into()
    }
}

impl From<SdCard> for VolumeMgr {
    fn from(value: SdCard) -> Self {
        let volume_mgr = embedded_sdmmc::VolumeManager::new(value.0, Clock);
        info!("has opened handles: {}", volume_mgr.has_open_handles());
        VolumeMgr::new(volume_mgr)
    }
}

#[derive(Debug)]
pub struct Clock;

impl TimeSource for Clock {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}
