use heapless::Vec;

use crate::peer::messages::{PeerMessageTypes, error::MessageError};
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PeerMessage<'a> {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),             // piece index
    BitField(Vec<u8, 64>), // bitfield data
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: &'a [u8], // block data
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
}

impl<'a> PeerMessage<'a> {
    pub(crate) fn get_type(&self) -> Option<u8> {
        match self {
            PeerMessage::Choke => Some(PeerMessageTypes::Choke as u8),
            PeerMessage::Unchoke => Some(PeerMessageTypes::Unchoke as u8),
            PeerMessage::Interested => Some(PeerMessageTypes::Interested as u8),
            PeerMessage::NotInterested => Some(PeerMessageTypes::NotInterested as u8),
            PeerMessage::Have(_) => Some(PeerMessageTypes::Have as u8),
            PeerMessage::BitField(_) => Some(PeerMessageTypes::Bitfield as u8),
            PeerMessage::Request { .. } => Some(PeerMessageTypes::Request as u8),
            PeerMessage::Piece { .. } => Some(PeerMessageTypes::Piece as u8),
            PeerMessage::Cancel { .. } => Some(PeerMessageTypes::Cancel as u8),
            PeerMessage::KeepAlive => None, // KeepAlive messages have no payload and no message type
        }
    }

    /// we will mostly send 13 bytes, only for the piece, more is required
    /// note that piece messages are currently not supported, let's see how we'll handle them
    /// TODO: piece messages
    pub(crate) fn as_bytes(&self) -> Vec<u8, 13> {
        let mut bytes = Vec::<u8, 13>::new();

        // first byte is the message type
        if let Some(message_type) = self.get_type() {
            bytes.push(message_type).expect("it's 13 bytes");
        } else {
            // KeepAlive message has no type and no payload, so we return an empty byte vector
            return bytes;
        }

        match self {
            PeerMessage::Choke
            | PeerMessage::Unchoke
            | PeerMessage::Interested
            | PeerMessage::NotInterested
            | PeerMessage::KeepAlive => {}
            PeerMessage::Have(piece_index) => {
                bytes.extend_from_slice(&piece_index.to_be_bytes()).unwrap();
            }
            PeerMessage::BitField(bitfield) => {
                bytes.extend_from_slice(bitfield).unwrap();
            }
            PeerMessage::Request {
                index,
                begin,
                length,
            } => {
                bytes.extend_from_slice(&index.to_be_bytes()).unwrap();
                bytes.extend_from_slice(&begin.to_be_bytes()).unwrap();
                bytes.extend_from_slice(&length.to_be_bytes()).unwrap();
            }
            PeerMessage::Cancel {
                index,
                begin,
                length,
            } => {
                bytes.extend_from_slice(&index.to_be_bytes()).unwrap();
                bytes.extend_from_slice(&begin.to_be_bytes()).unwrap();
                bytes.extend_from_slice(&length.to_be_bytes()).unwrap();
            }
            PeerMessage::Piece { .. } => unimplemented!("Piece messages are not supported yet"),
        }

        bytes
    }

    pub(crate) fn from_bytes(data: &'a [u8]) -> Result<Self, MessageError> {
        if data.is_empty() {
            return Ok(Self::KeepAlive);
        }
        match data[0] {
            b if b == PeerMessageTypes::Choke as u8 => Ok(PeerMessage::Choke),
            b if b == PeerMessageTypes::Unchoke as u8 => Ok(PeerMessage::Unchoke),
            b if b == PeerMessageTypes::Interested as u8 => Ok(PeerMessage::Interested),
            b if b == PeerMessageTypes::NotInterested as u8 => Ok(PeerMessage::NotInterested),
            b if b == PeerMessageTypes::Have as u8 => parse_have_message(data),
            b if b == PeerMessageTypes::Bitfield as u8 => parse_bitfield_message(data),
            b if b == PeerMessageTypes::Request as u8 => parse_request_message(data),
            b if b == PeerMessageTypes::Piece as u8 => parse_piece_message(data),
            b if b == PeerMessageTypes::Cancel as u8 => parse_cancel_message(data),
            b => Err(MessageError::UnknownMessageType(b)),
        }
    }
}

fn parse_have_message<'a>(data: &'a [u8]) -> Result<PeerMessage<'a>, MessageError> {
    if data.len() < 5 {
        return Err(MessageError::InvalidLength);
    }
    let piece_index = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
    Ok(PeerMessage::Have(piece_index))
}

fn parse_bitfield_message<'a>(_data: &'a [u8]) -> Result<PeerMessage<'a>, MessageError> {
    todo!("Bitfield message parsing not implemented yet");
}

fn parse_request_message<'a>(data: &'a [u8]) -> Result<PeerMessage<'a>, MessageError> {
    if data.len() < 13 {
        return Err(MessageError::InvalidLength);
    }

    let index = u32::from_be_bytes(data[1..=4].try_into().unwrap());
    let begin = u32::from_be_bytes(data[5..=8].try_into().unwrap());
    let length = u32::from_be_bytes(data[9..=12].try_into().unwrap());

    Ok(PeerMessage::Request {
        index,
        begin,
        length,
    })
}

fn parse_piece_message<'a>(data: &'a [u8]) -> Result<PeerMessage<'a>, MessageError> {
    if data.len() < 13 {
        return Err(MessageError::InvalidLength);
    }

    let index = u32::from_be_bytes(data[1..=4].try_into().unwrap());
    let begin = u32::from_be_bytes(data[5..=8].try_into().unwrap());
    let block_data = &data[9..];

    Ok(PeerMessage::Piece {
        index,
        begin,
        block: block_data,
    })
}

fn parse_cancel_message<'a>(data: &'a [u8]) -> Result<PeerMessage<'a>, MessageError> {
    if data.len() < 13 {
        return Err(MessageError::InvalidLength);
    }

    let index = u32::from_be_bytes(data[1..=4].try_into().unwrap());
    let begin = u32::from_be_bytes(data[5..=8].try_into().unwrap());
    let length = u32::from_be_bytes(data[9..=12].try_into().unwrap());

    Ok(PeerMessage::Cancel {
        index,
        begin,
        length,
    })
}

impl<'a> TryInto<u8> for PeerMessage<'a> {
    type Error = ();

    fn try_into(self) -> Result<u8, Self::Error> {
        self.get_type().ok_or(())
    }
}
