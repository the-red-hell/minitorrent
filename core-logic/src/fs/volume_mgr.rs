use core::ops::Deref;
use embedded_sdmmc::{BlockDevice, RawDirectory, RawVolume, TimeSource, VolumeManager};

pub trait VolumeMgr: Deref<Target = VolumeManager<Self::B, Self::T>>
where
    <Self::B as BlockDevice>::Error: core::fmt::Debug,
{
    type B: BlockDevice;
    type T: TimeSource;

    fn new(vol_mgr: VolumeManager<Self::B, Self::T>) -> Self;

    fn get_vol0(&self) -> RawVolume;

    fn get_root_dir(&self, volume: RawVolume) -> RawDirectory;
}
