//! Peer message definitions and related traits.

pub(crate) mod error;

use crate::peer::{buf_reader::BufReader, messages::error::MessageError};

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

#[defmt_or_log::derive_format_or_debug]
pub enum PeerMessage<'a> {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32), // piece index
    BitField(#[cfg_attr(feature = "defmt", defmt(Debug2Format))] alloc::vec::Vec<bool>), // bitfield data
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
    pub(crate) const fn get_type(&self) -> Option<u8> {
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

    /// note that piece messages are currently not supported, let's see how we'll handle them
    /// TODO: piece messages
    pub(crate) fn as_bittorrent_bytes(&self) -> alloc::vec::Vec<u8> {
        // we will mostly send 17 bytes, only for the piece, more is required
        let mut bytes = alloc::vec::Vec::with_capacity(17);

        // --- message length
        let length: u32 = match self {
            PeerMessage::KeepAlive => 0,
            PeerMessage::Choke
            | PeerMessage::Unchoke
            | PeerMessage::Interested
            | PeerMessage::NotInterested => 1,
            PeerMessage::Have(_) => 5,
            PeerMessage::BitField(bitfield) => {
                let bitfield_len = bitfield.len().div_ceil(8);
                // reserve needed bytes for bitfield payload
                bytes.reserve(bitfield_len - bytes.len());
                bitfield_len as u32 + 1 // +1 for the message type
            }
            PeerMessage::Request { .. } | PeerMessage::Cancel { .. } => 13,
            PeerMessage::Piece { .. } => unimplemented!("Piece messages are not supported yet"), // + 7
        };

        bytes.extend_from_slice(&length.to_be_bytes());

        // --- message type
        if let Some(message_type) = self.get_type() {
            bytes.push(message_type);
        } else {
            // KeepAlive message has no type and no payload, so we return an empty byte vector
            return bytes;
        }

        // --- payload
        match self {
            PeerMessage::Choke
            | PeerMessage::Unchoke
            | PeerMessage::Interested
            | PeerMessage::NotInterested
            | PeerMessage::KeepAlive => {}
            PeerMessage::Have(piece_index) => {
                bytes.extend_from_slice(&piece_index.to_be_bytes());
            }
            PeerMessage::BitField(bitfield) => {
                bitfield
                    .chunks(8)
                    .map(|chunk| {
                        let mut byte = 0u8;
                        for (i, &have) in chunk.iter().enumerate() {
                            if have {
                                byte |= 128 >> i;
                            }
                        }
                        byte
                    })
                    .for_each(|byte| bytes.push(byte));
            }
            PeerMessage::Request {
                index,
                begin,
                length,
            } => {
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(&length.to_be_bytes());
            }
            PeerMessage::Cancel {
                index,
                begin,
                length,
            } => {
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(&length.to_be_bytes());
            }
            PeerMessage::Piece { .. } => unimplemented!("Piece messages are not supported yet"),
        }

        bytes
    }

    pub(crate) fn from_bytes<const CAP: usize>(
        data: &'a mut BufReader<CAP>,
    ) -> Result<Option<Self>, MessageError> {
        if data.len() < 4 {
            return Ok(None); // Not enough data to read
        }

        let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if len == 0 {
            return Ok(Some(Self::KeepAlive));
        }

        let payload = &data.as_slice()[4..];

        if len as usize > payload.len() {
            return Ok(None); // Not enough data for the full message
        }

        match payload[0] {
            b if len == 1 && b == PeerMessageTypes::Choke as u8 => Ok(Some(PeerMessage::Choke)),
            b if len == 1 && b == PeerMessageTypes::Unchoke as u8 => Ok(Some(PeerMessage::Unchoke)),
            b if len == 1 && b == PeerMessageTypes::Interested as u8 => {
                Ok(Some(PeerMessage::Interested))
            }
            b if len == 1 && b == PeerMessageTypes::NotInterested as u8 => {
                Ok(Some(PeerMessage::NotInterested))
            }
            b if b == PeerMessageTypes::Have as u8 => parse_have_message(payload),
            b if b == PeerMessageTypes::Bitfield as u8 => parse_bitfield_message(payload),
            b if b == PeerMessageTypes::Request as u8 => parse_request_message(payload),
            b if b == PeerMessageTypes::Piece as u8 => parse_piece_message(payload),
            b if b == PeerMessageTypes::Cancel as u8 => parse_cancel_message(payload),
            b => Err(MessageError::UnknownMessageType(b)),
        }
    }
}

const fn parse_have_message<'a>(data: &'a [u8]) -> Result<Option<PeerMessage<'a>>, MessageError> {
    if data.len() < 5 {
        return Err(MessageError::InvalidLength);
    }
    let piece_index = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
    Ok(Some(PeerMessage::Have(piece_index)))
}

fn parse_bitfield_message<'a>(data: &'a [u8]) -> Result<Option<PeerMessage<'a>>, MessageError> {
    let mut have = alloc::vec::Vec::from_iter(::core::iter::repeat_n(false, (data.len() - 1) * 8));

    for (byte_i, &byte) in data[1..].iter().enumerate() {
        for bit_i in 0..8 {
            let byte_offset = 128 >> bit_i;
            let do_we_have = (byte & byte_offset) == byte_offset;
            have[byte_i * 8 + bit_i] = do_we_have;
        }
    }

    Ok(Some(PeerMessage::BitField(have)))
}

const fn parse_request_message<'a>(
    data: &'a [u8],
) -> Result<Option<PeerMessage<'a>>, MessageError> {
    if data.len() < 13 {
        return Err(MessageError::InvalidLength);
    }

    let index = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
    let begin = u32::from_be_bytes([data[5], data[6], data[7], data[8]]);
    let length = u32::from_be_bytes([data[9], data[10], data[11], data[12]]);

    Ok(Some(PeerMessage::Request {
        index,
        begin,
        length,
    }))
}

fn parse_piece_message<'a>(data: &'a [u8]) -> Result<Option<PeerMessage<'a>>, MessageError> {
    if data.len() < 13 {
        return Err(MessageError::InvalidLength);
    }

    let index = u32::from_be_bytes(data[1..=4].try_into().unwrap());
    let begin = u32::from_be_bytes(data[5..=8].try_into().unwrap());
    let block_data = &data[9..];

    Ok(Some(PeerMessage::Piece {
        index,
        begin,
        block: block_data,
    }))
}

fn parse_cancel_message<'a>(data: &'a [u8]) -> Result<Option<PeerMessage<'a>>, MessageError> {
    if data.len() < 13 {
        return Err(MessageError::InvalidLength);
    }

    let index = u32::from_be_bytes(data[1..=4].try_into().unwrap());
    let begin = u32::from_be_bytes(data[5..=8].try_into().unwrap());
    let length = u32::from_be_bytes(data[9..=12].try_into().unwrap());

    Ok(Some(PeerMessage::Cancel {
        index,
        begin,
        length,
    }))
}

impl<'a> TryInto<u8> for PeerMessage<'a> {
    type Error = ();

    #[inline]
    fn try_into(self) -> Result<u8, Self::Error> {
        self.get_type().ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_enough_data() {
        let mut buf = BufReader::<10>::new();
        buf.remaining_mut()[..3].copy_from_slice(&[0, 0, 0]); // only 3 bytes, should be at least 4 for length
        assert!(PeerMessage::from_bytes(&mut buf).unwrap().is_none());
    }

    #[test]
    fn test_keep_alive_message() {
        let mut buf = BufReader::<10>::new();
        buf.remaining_mut()[..4].copy_from_slice(&0u32.to_be_bytes());
        buf.advance_n(4);

        let msg = PeerMessage::from_bytes(&mut buf).unwrap().unwrap();

        assert!(matches!(msg, PeerMessage::KeepAlive));
        assert_eq!(msg.as_bittorrent_bytes().as_slice(), &vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_singular_messages() {
        // Choke, Unchoke, Interested, NotInterested

        // Choke
        let mut buf = BufReader::<10>::new();
        buf.remaining_mut()[..4].copy_from_slice(&1u32.to_be_bytes());
        buf.remaining_mut()[4] = PeerMessageTypes::Choke as u8;
        buf.advance_n(5);

        let msg = PeerMessage::from_bytes(&mut buf).unwrap().unwrap();

        assert!(matches!(msg, PeerMessage::Choke));
        assert_eq!(
            msg.as_bittorrent_bytes().as_slice(),
            &vec![0, 0, 0, 1, PeerMessageTypes::Choke as u8]
        );

        // Unchoke
        buf.reset();
        buf.remaining_mut()[..4].copy_from_slice(&1u32.to_be_bytes());
        buf.remaining_mut()[4] = PeerMessageTypes::Unchoke as u8;
        buf.advance_n(5);

        let msg = PeerMessage::from_bytes(&mut buf).unwrap().unwrap();

        assert!(matches!(msg, PeerMessage::Unchoke));
        assert_eq!(
            msg.as_bittorrent_bytes().as_slice(),
            &vec![0, 0, 0, 1, PeerMessageTypes::Unchoke as u8]
        );

        // Interested
        buf.reset();
        buf.remaining_mut()[..4].copy_from_slice(&1u32.to_be_bytes());
        buf.remaining_mut()[4] = PeerMessageTypes::Interested as u8;
        buf.advance_n(5);

        let msg = PeerMessage::from_bytes(&mut buf).unwrap().unwrap();

        assert!(matches!(msg, PeerMessage::Interested));
        assert_eq!(
            msg.as_bittorrent_bytes().as_slice(),
            &vec![0, 0, 0, 1, PeerMessageTypes::Interested as u8]
        );

        // NotInterested
        buf.reset();
        buf.remaining_mut()[..4].copy_from_slice(&1u32.to_be_bytes());
        buf.remaining_mut()[4] = PeerMessageTypes::NotInterested as u8;
        buf.advance_n(5);

        let msg = PeerMessage::from_bytes(&mut buf).unwrap().unwrap();

        assert!(matches!(msg, PeerMessage::NotInterested));
        assert_eq!(
            msg.as_bittorrent_bytes().as_slice(),
            &vec![0, 0, 0, 1, PeerMessageTypes::NotInterested as u8]
        );
    }

    #[test]
    fn test_have_message() {
        let mut buf = BufReader::<10>::new();
        let piece_index = 12345u32.to_be_bytes();
        buf.remaining_mut()[..4].copy_from_slice(&(piece_index.len() as u32 + 1).to_be_bytes());
        buf.remaining_mut()[4] = PeerMessageTypes::Have as u8;
        buf.remaining_mut()[5..9].copy_from_slice(&piece_index);
        buf.advance_n(9);

        let msg = PeerMessage::from_bytes(&mut buf).unwrap().unwrap();

        assert!(matches!(msg, PeerMessage::Have(12345)));
        assert_eq!(
            msg.as_bittorrent_bytes().as_slice(),
            [
                &[0, 0, 0, 5, PeerMessageTypes::Have as u8][..],
                &piece_index
            ]
            .concat()
        );
    }

    #[test]
    fn test_bitfield_message() {
        let mut buf = BufReader::<10>::new();
        let have = vec![
            true, false, true, true, false, true, true, true, false, false, false, false, false,
            false, false, false,
        ];
        let expected_bitfield_bytes = vec![0b10110111, 0];
        buf.remaining_mut()[..4].copy_from_slice(2u32.to_be_bytes().as_slice());
        buf.remaining_mut()[4] = PeerMessageTypes::Bitfield as u8;
        buf.remaining_mut()[5..7].copy_from_slice(&expected_bitfield_bytes);
        buf.advance_n(7);

        let msg = PeerMessage::from_bytes(&mut buf).unwrap().unwrap();

        assert!(matches!(msg, PeerMessage::BitField(recv_have) if recv_have == have));

        assert_eq!(
            PeerMessage::BitField(have).as_bittorrent_bytes(),
            [
                &[0, 0, 0, 3, PeerMessageTypes::Bitfield as u8][..],
                &expected_bitfield_bytes
            ]
            .concat()
        );
    }

    #[test]
    fn test_request_message() {
        let mut buf = BufReader::<20>::new();
        let index = 1u32.to_be_bytes();
        let begin = 2u32.to_be_bytes();
        let length = 3u32.to_be_bytes();
        buf.remaining_mut()[..4].copy_from_slice(&13u32.to_be_bytes());
        buf.remaining_mut()[4] = PeerMessageTypes::Request as u8;
        buf.remaining_mut()[5..9].copy_from_slice(&index);
        buf.remaining_mut()[9..13].copy_from_slice(&begin);
        buf.remaining_mut()[13..17].copy_from_slice(&length);
        buf.advance_n(17);

        let msg = PeerMessage::from_bytes(&mut buf).unwrap().unwrap();

        assert!(matches!(
            msg,
            PeerMessage::Request {
                index: 1,
                begin: 2,
                length: 3
            }
        ));
        assert_eq!(
            msg.as_bittorrent_bytes().as_slice(),
            [
                &[0, 0, 0, 13, PeerMessageTypes::Request as u8][..],
                &index,
                &begin,
                &length
            ]
            .concat()
        )
    }

    #[test]
    fn test_piece_message() {
        let mut buf = BufReader::<20>::new();
        let index = 1u32.to_be_bytes();
        let begin = 2u32.to_be_bytes();
        let block = vec![0u8; 4];
        buf.remaining_mut()[..4].copy_from_slice(&13u32.to_be_bytes());
        buf.remaining_mut()[4] = PeerMessageTypes::Piece as u8;
        buf.remaining_mut()[5..9].copy_from_slice(&index);
        buf.remaining_mut()[9..13].copy_from_slice(&begin);
        buf.remaining_mut()[13..17].copy_from_slice(&block);
        buf.advance_n(17);

        let msg = PeerMessage::from_bytes(&mut buf);

        assert!(matches!(
            msg,
            Ok(Some(PeerMessage::Piece {
                index: 1,
                begin: 2,
                block: &[0, 0, 0, 0]
            }))
        ),);

        // TODO: test block
        // assert_eq!(
        //     msg.as_bittorrent_bytes().as_slice(),
        //     [
        //         &[0, 0, 0, 17, PeerMessageTypes::Piece as u8][..],
        //         &index,
        //         &begin,
        //         &block
        //     ]
        //     .concat()
        // )
    }
}
