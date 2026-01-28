use core::ops::Deref;
use embedded_sdmmc::{BlockDevice, RawDirectory, RawVolume, TimeSource, VolumeManager};

pub trait VolumeMgr: Deref<Target = VolumeManager<Self::BlockDevice, Self::TimeSource>>
where
    <Self::BlockDevice as BlockDevice>::Error: core::fmt::Debug,
{
    type BlockDevice: BlockDevice;
    type TimeSource: TimeSource;

    fn new(vol_mgr: VolumeManager<Self::BlockDevice, Self::TimeSource>) -> Self;

    fn get_vol0(&self) -> RawVolume;

    fn get_root_dir(&self, volume: RawVolume) -> RawDirectory;
}
