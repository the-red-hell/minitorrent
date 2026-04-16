use embedded_sdmmc::ShortFileName;
use heapless::Vec;

use crate::{MetaInfoFile, core::InfoHash};

pub struct RequestingTracker;

#[cfg_attr(feature = "log", derive(Debug))]
pub struct Downloading {
    peers: Vec<core::net::SocketAddrV4, 10>,
    info_hash: InfoHash,
    piece_length: u32,
    total_length: u32,
    name: ShortFileName,
    piece_hashes: alloc::vec::Vec<InfoHash>,
}

impl Downloading {
    pub(crate) fn new(
        peers: Vec<core::net::SocketAddrV4, 10>,
        metainfo: &MetaInfoFile<'_>,
    ) -> Self {
        let name = ShortFileName::create_from_str(metainfo.info.name)
            .unwrap_or_else(|_| ShortFileName::create_from_str("1").expect("is valid")); // TODO: maybe generate uuid or counting up
        Self {
            peers,
            info_hash: metainfo.info_hash,
            piece_length: metainfo.info.piece_length,
            total_length: metainfo.info.length,
            name,
            piece_hashes: metainfo.info.pieces.to_vec(),
        }
    }
    pub fn get_info_hash(&self) -> &InfoHash {
        &self.info_hash
    }
    pub fn get_total_length(&self) -> u32 {
        self.total_length
    }

    pub fn get_name(&self) -> &ShortFileName {
        &self.name
    }

    pub(crate) fn get_piece_length(&self) -> u32 {
        self.piece_length
    }

    pub(crate) fn get_peers(&self) -> &[core::net::SocketAddrV4] {
        &self.peers
    }

    pub(crate) fn get_pieces_hashes(&self) -> &[InfoHash] {
        &self.piece_hashes
    }
}

#[defmt_or_log::derive_format_or_debug]
pub struct _Seeding;
