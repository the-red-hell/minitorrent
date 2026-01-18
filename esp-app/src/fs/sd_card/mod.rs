pub mod volume;

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

use crate::fs::sd_card::volume::VolumeMgr;

#[derive(Debug)]
pub enum SdCardError {
    SPIError,
}

pub(super) struct SdCard<'a>(
    embedded_sdmmc::SdCard<ExclusiveDevice<Spi<'a, Blocking>, gpio::Output<'a>, Delay>, Delay>,
);

/// A struct holding all the data of the physically connected pins to the SD Card module.
pub struct SPIInitializer<'a, SCK, MISO, MOSI, CS>
where
    SCK: gpio::interconnect::PeripheralOutput<'a>,
    MISO: gpio::interconnect::PeripheralInput<'a>,
    MOSI: gpio::interconnect::PeripheralOutput<'a>,
    CS: gpio::OutputPin,
{
    sck: SCK,
    miso: MISO,
    mosi: MOSI,
    cs: CS,
    _phantom: PhantomData<&'a bool>,
}
impl<'a, SCK, MISO, MOSI, CS> SPIInitializer<'a, SCK, MISO, MOSI, CS>
where
    SCK: gpio::interconnect::PeripheralOutput<'a>,
    MISO: gpio::interconnect::PeripheralInput<'a>,
    MOSI: gpio::interconnect::PeripheralOutput<'a>,
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

impl<'a> SdCard<'a> {
    /// Initializes the SD Card.
    pub(super) fn init<'b, SPI, SCK, MISO, MOSI, CS>(
        initializer: SPIInitializer<'b, SCK, MISO, MOSI, CS>,
        spi: SPI,
    ) -> Result<Self, SdCardError>
    where
        'b: 'a,
        SPI: spi::master::Instance + 'b,
        SCK: gpio::interconnect::PeripheralOutput<'b>,
        MISO: gpio::interconnect::PeripheralInput<'b>,
        MOSI: gpio::interconnect::PeripheralOutput<'b>,
        CS: gpio::OutputPin + 'b,
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

    pub(super) fn to_volume_mgr(self) -> VolumeMgr<'a> {
        self.into()
    }
}

impl<'a> From<SdCard<'a>> for VolumeMgr<'a> {
    fn from(value: SdCard<'a>) -> Self {
        let volume_mgr = embedded_sdmmc::VolumeManager::new(value.0, Clock);
        info!("has opened handles: {}", volume_mgr.has_open_handles());
        VolumeMgr(volume_mgr)
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
