use core::ops::Deref;

use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{SdCard, VolumeManager};
use esp_hal::{Blocking, delay::Delay, gpio, spi::master::Spi};

type VolumeMgrType<'a> = VolumeManager<
    SdCard<ExclusiveDevice<Spi<'a, Blocking>, gpio::Output<'a>, Delay>, Delay>,
    super::Clock,
>;
type VolumeType<'a> = embedded_sdmmc::Volume<
    'a,
    SdCard<ExclusiveDevice<Spi<'a, Blocking>, gpio::Output<'a>, Delay>, Delay>,
    super::Clock,
    4,
    4,
    1,
>;

pub(in crate::fs) struct VolumeMgr<'a>(pub(in crate::fs) VolumeMgrType<'a>);

unsafe impl<'a> Sync for VolumeMgr<'a> {}

#[derive(Debug)]
pub struct Volume<'a>(VolumeType<'a>);

// #[derive(Debug)]
// pub struct RootDir<'a>(
//     Directory<
//         'a,
//         SdCard<ExclusiveDevice<Spi<'a, Blocking>, gpio::Output<'a>, Delay>, Delay>,
//         super::Clock,
//         4,
//         4,
//         1,
//     >,
// );

impl<'a> VolumeMgr<'a> {
    pub(in crate::fs) fn get_volume(&'a self) -> Volume<'a> {
        loop {
            match self.0.open_volume(embedded_sdmmc::VolumeIdx(0)) {
                Ok(volume0) => break Volume(volume0),
                Err(e) => {
                    defmt::warn!("failed to open volume 0 with error {:?}", e);
                    Delay::new().delay_millis(1000);
                }
            }
        }
    }
}

// impl<'a> Volume<'a> {
//     pub fn open_root_dir(&'a self) -> RootDir<'a> {
//         loop {
//             match self.0.open_root_dir() {
//                 Ok(root_dir) => break RootDir(root_dir),
//                 Err(e) => {
//                     defmt::warn!("failed to open root_dir with error {:?}", e);
//                     Delay::new().delay_millis(1000);
//                 }
//             }
//         }
//     }
// }

impl<'a> Deref for Volume<'a> {
    type Target = VolumeType<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
