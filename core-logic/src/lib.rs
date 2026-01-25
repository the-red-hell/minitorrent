#![cfg_attr(not(test), no_std)]

use crate::{fs::FileSystem, wifi::WifiStack};

pub mod fs;
pub mod metainfo;
pub mod wifi;

pub struct BitTorrenter<WIFI, FS>
where
    WIFI: WifiStack,
    FS: FileSystem,
{
    wifi: WIFI,
    fs: FS,
}

impl<WIFI, FS> BitTorrenter<WIFI, FS>
where
    WIFI: WifiStack,
    FS: FileSystem,
{
    pub fn new(wifi: WIFI, fs: FS) -> Self {
        Self { wifi, fs }
    }

    pub fn fs(&mut self) -> &mut FS {
        &mut self.fs
    }

    pub fn wifi(&mut self) -> &mut WIFI {
        &mut self.wifi
    }
}
