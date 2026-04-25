//! Peer message definitions and related traits.

pub mod error;
pub(in crate::peer) mod messages;

#[repr(u8)]
pub(crate) enum PeerMessageTypes {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}
