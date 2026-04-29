#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod bittorrenter;
pub mod core;
pub mod fs;
pub mod hash;
pub mod net;
mod peer;

pub use bittorrenter::{BitTorrenter, error::BitTorrenterError};
pub use core::metainfo::{Info, MetaInfoFile};
pub use hash::Sha1Hasher;
pub use net::tcp::TcpConnector;
pub use peer::BLOCK_SIZE;
pub use peer::messages::error::MessageError;
