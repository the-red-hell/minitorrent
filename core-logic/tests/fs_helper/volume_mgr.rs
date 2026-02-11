use std::ops::Deref;

use core_logic::fs::VolumeMgr;
use embedded_sdmmc::VolumeManager;

use crate::fs_helper::blockdevice::{Clock, LinuxBlockDevice};

pub type VolumeMgrType = VolumeManager<LinuxBlockDevice, Clock>;

#[derive(Debug)]
pub struct VolumeMgrDuple(pub VolumeMgrType);

impl VolumeMgr for VolumeMgrDuple {
    type BlockDevice = LinuxBlockDevice;

    type TimeSource = Clock;

    fn new(vol_mgr: VolumeManager<Self::BlockDevice, Self::TimeSource>) -> Self {
        Self(vol_mgr)
    }

    fn get_vol0(&self) -> embedded_sdmmc::RawVolume {
        match self.0.open_volume(embedded_sdmmc::VolumeIdx(0)) {
            Ok(volume0) => return volume0.to_raw_volume(),
            Err(e) => {
                panic!("failed to open volume 0 with error {:?}", e);
            }
        }
    }

    fn get_root_dir(&self, volume: embedded_sdmmc::RawVolume) -> embedded_sdmmc::RawDirectory {
        match volume.to_volume(&self.0).open_root_dir() {
            Ok(root_dir) => return root_dir.to_raw_directory(),
            Err(e) => {
                panic!("failed to open root directory with error {:?}", e);
            }
        }
    }
}

impl Deref for VolumeMgrDuple {
    type Target = VolumeManager<LinuxBlockDevice, Clock>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
