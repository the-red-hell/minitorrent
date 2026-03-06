use alloc::vec::Vec;

use crate::MetaInfoFile;

pub struct RequestingTracker;

#[derive(Debug)]
pub struct Downloading {
    peers: heapless::Vec<core::net::SocketAddrV4, 10>,
    info_hash: [u8; 20],
    piece_length: u32,
    total_length: u32,
    pieces: Vec<[u8; 20]>,
}
impl Downloading {
    pub(crate) fn new(
        peers: heapless::Vec<core::net::SocketAddrV4, 10>,
        metainfo: &MetaInfoFile<'_>,
    ) -> Self {
        Self {
            peers,
            info_hash: metainfo.info_hash,
            piece_length: metainfo.info.piece_length,
            total_length: metainfo.info.length,
            pieces: metainfo.info.pieces.to_vec(),
        }
    }

    pub(crate) fn get_peers(&self) -> &[core::net::SocketAddrV4] {
        &self.peers
    }

    pub(crate) fn get_info_hash(&self) -> &[u8; 20] {
        &self.info_hash
    }

    pub(crate) fn get_piece_length(&self) -> u32 {
        self.piece_length
    }

    pub(crate) fn get_total_length(&self) -> u32 {
        self.total_length
    }
}

#[derive(Debug)]
pub struct _Seeding;
