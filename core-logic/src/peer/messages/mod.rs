//! Peer message definitions and related traits.
//! Use `PeerMessage` enum to identify message types and `Message<MSG>` struct to wrap message data.
//! You must implement `MessageType` for each message struct to specify its type.

pub mod error;
pub(in crate::peer) mod incoming;

// use self::{
//     choking::{Choke, Unchoke},
//     have::Have,
//     interest::{Interested, NotInterested},
//     // piece::Piece,
//     request::Request,
// };

// pub(crate) struct Message<MSG: MessageType>(MSG);

// /// This trait must be implemented for each message struct to specify its type (e.g., Choke, Interested, Request, etc.).
// pub(crate) trait MessageType {
//     type Msg;

//     fn get_type(&self) -> u8;
// }

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

// impl<MSG: MessageType> Message<MSG> {
//     pub fn new(msg: MSG) -> Self {
//         Self(msg)
//     }

//     pub fn get_type(&self) -> u8 {
//         MSG::get_type(&self.0)
//     }

//     pub fn into_inner(self) -> MSG {
//         self.0
//     }

//     // into_bytes?
// }

// pub(crate) mod request {
//     use super::*;

//     pub(crate) struct Request {
//         pub index: u32,
//         pub begin: u32,
//         pub length: u32,
//     }

//     impl MessageType for Request {
//         type Msg = Self;

//         fn get_type(&self) -> u8 {
//             PeerMessageTypes::Request as u8
//         }
//     }

//     impl Request {
//         pub(crate) fn new(index: u32, begin: u32, length: u32) -> Self {
//             Self {
//                 index,
//                 begin,
//                 length,
//             }
//         }
//     }
// }

// pub(crate) mod choking {
//     use super::*;

//     pub(crate) struct Choke;

//     impl MessageType for Choke {
//         type Msg = Self;

//         fn get_type(&self) -> u8 {
//             PeerMessageTypes::Choke as u8
//         }
//     }

//     pub(crate) struct Unchoke;

//     impl MessageType for Unchoke {
//         type Msg = Self;

//         fn get_type(&self) -> u8 {
//             PeerMessageTypes::Unchoke as u8
//         }
//     }
// }

// pub(crate) mod interest {
//     use super::*;

//     pub(crate) struct Interested;

//     impl MessageType for Interested {
//         type Msg = Self;

//         fn get_type(&self) -> u8 {
//             PeerMessageTypes::Interested as u8
//         }
//     }

//     pub(crate) struct NotInterested;

//     impl MessageType for NotInterested {
//         type Msg = Self;

//         fn get_type(&self) -> u8 {
//             PeerMessageTypes::NotInterested as u8
//         }
//     }
// }

// pub(crate) mod have {
//     use super::*;

//     pub(crate) struct Have {
//         piece_index: u32,
//     }

//     impl MessageType for Have {
//         type Msg = Self;

//         fn get_type(&self) -> u8 {
//             PeerMessageTypes::Have as u8
//         }
//     }

//     impl Have {
//         pub(crate) fn new(piece_index: u32) -> Self {
//             Self { piece_index }
//         }
//     }
// }

// // pub(crate) mod piece {
// //     use crate::peer::BLOCK_SIZE;

// //     use super::*;

// //     pub(crate) struct Piece {
// //         index: u32,
// //         begin: u32,
// //         block: heapless::Vec<u8, BLOCK_SIZE>,
// //     }

// //     impl MessageType for Piece {
// //         type Msg = Self;

// //         fn get_type(&self) -> u8 {
// //             PeerMessage::Piece as u8
// //         }
// //     }

// //     impl Piece {
// //         /// **Clones** the block data into the message struct, ensuring it does not exceed the maximum block size.
// //         pub(crate) fn new(index: u32, begin: u32, block: &[u8]) -> Self {
// //             // TODO: SHA1
// //             let mut block_vec = heapless::Vec::<u8, BLOCK_SIZE>::new();
// //             block_vec
// //                 .extend_from_slice(block)
// //                 .expect("Block size exceeds maximum");
// //             Self {
// //                 index,
// //                 begin,
// //                 block: block_vec,
// //             }
// //         }
// //     }
// // }
