#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod bittorrenter;
pub mod core;
pub mod fs;
pub mod net;

pub use bittorrenter::{BitTorrenter, error::BitTorrenterError};
pub use core::metainfo::{Info, MetaInfoFile};
pub use net::tcp::TcpConnector;
