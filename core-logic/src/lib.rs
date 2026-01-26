#![cfg_attr(not(test), no_std)]

extern crate alloc;

use crate::{
    fs::{FileSystem, VolumeMgr},
    wifi::WifiStack,
};

pub mod fs;
pub mod metainfo;
pub mod wifi;

pub struct BitTorrenter<WIFI, V>
where
    WIFI: WifiStack,
    V: VolumeMgr,
{
    wifi: WIFI,
    fs: FileSystem<V>,
}

impl<WIFI, V> BitTorrenter<WIFI, V>
where
    WIFI: WifiStack,
    V: VolumeMgr,
{
    pub fn new(wifi: WIFI, fs: FileSystem<V>) -> Self {
        Self { wifi, fs }
    }

    pub fn fs(&mut self) -> &mut FileSystem<V> {
        &mut self.fs
    }

    pub fn wifi(&mut self) -> &mut WIFI {
        &mut self.wifi
    }
}
