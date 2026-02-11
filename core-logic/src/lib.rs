#![cfg_attr(not(test), no_std)]

extern crate alloc;

use embedded_nal_async::{Dns, TcpConnect};
use embedded_sdmmc::BlockDevice;

use crate::fs::{FileSystem, VolumeMgr};

pub mod core;
pub mod fs;
// pub mod wifi;

pub use core::metainfo::{Info, MetaInfoFile};

pub struct BitTorrenter<NET, V>
where
    NET: TcpConnect + Dns,
    V: VolumeMgr,
{
    net: NET,
    fs: FileSystem<V>,
    peer_id: [u8; 20],
    port: u16,
}

impl<NET, V> BitTorrenter<NET, V>
where
    NET: TcpConnect + Dns,
    V: VolumeMgr,
{
    pub fn new(net: NET, fs: FileSystem<V>) -> Self {
        Self {
            net,
            fs,
            peer_id: [0u8; 20],
            port: 6881,
        }
    }

    pub fn fs(&mut self) -> &mut FileSystem<V> {
        &mut self.fs
    }

    pub fn net(&mut self) -> &mut NET {
        &mut self.net
    }
}

#[derive(Debug)]
pub enum BitTorrenterError<NET, V>
where
    NET: TcpConnect + Dns,
    V: VolumeMgr,
{
    DnsError(<NET as Dns>::Error),
    TcpError(<NET as TcpConnect>::Error),
    FsError(<<V as VolumeMgr>::BlockDevice as BlockDevice>::Error),
}
