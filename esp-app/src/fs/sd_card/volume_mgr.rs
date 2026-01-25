use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{RawDirectory, RawVolume, SdCard, VolumeManager};
use esp_hal::{Blocking, delay::Delay, gpio, spi::master::Spi};

pub(in crate::fs) type VolumeMgrType<'a> = VolumeManager<
    SdCard<ExclusiveDevice<Spi<'a, Blocking>, gpio::Output<'a>, Delay>, Delay>,
    super::Clock,
>;

pub(in crate::fs) struct VolumeMgr(pub(in crate::fs) VolumeMgrType<'static>);

impl VolumeMgr {
    pub(in crate::fs) fn new(mgr: VolumeMgrType<'static>) -> Self {
        Self(mgr)
    }

    pub(in crate::fs) fn get_vol0(&self) -> RawVolume {
        loop {
            match self.0.open_volume(embedded_sdmmc::VolumeIdx(0)) {
                Ok(volume0) => break volume0.to_raw_volume(),
                Err(e) => {
                    defmt::warn!("failed to open volume 0 with error {:?}", e);
                    Delay::new().delay_millis(1000);
                }
            }
        }
    }

    pub(in crate::fs) fn get_root_dir(&self, volume: RawVolume) -> RawDirectory {
        loop {
            match volume.to_volume(&self.0).open_root_dir() {
                Ok(root_dir) => break root_dir.to_raw_directory(),
                Err(e) => {
                    defmt::warn!("failed to open root_dir with error {:?}", e);
                    Delay::new().delay_millis(1000);
                }
            }
        }
    }
}
