use bencode::{BencodeParser, Error, Result};
use defmt::Format;

use crate::core::InfoHash;

#[derive(Debug, PartialEq, Format)]
pub struct MetaInfoFile<'a> {
    pub announce: &'a str,
    pub info: Info<'a>,
    pub info_hash: [u8; 20],
}

#[derive(Debug, PartialEq, Format)]
pub struct Info<'a> {
    pub piece_length: u32,
    pub name: &'a str,
    pub pieces: &'a [InfoHash],
    pub length: u32,
}

impl<'a> MetaInfoFile<'a> {
    pub fn parse(input: &'a [u8]) -> Result<Self> {
        let mut p = BencodeParser::new(input);

        // Prepare default values (Option is useful here if fields are optional)
        let mut announce = None;
        let mut info = None;
        let mut info_hash = [0u8; 20];

        p.expect_dict_start()?;

        // Dictionary Loop
        while !p.match_dict_end() {
            let key = p.parse_str()?; // Keys are always strings

            match key {
                "announce" => {
                    announce = Some(p.parse_str()?);
                }
                "info" => {
                    let info_bytes = p.parse_raw_value()?;
                    info_hash = sha1_smol::Sha1::from(info_bytes).digest().bytes();
                    info = Some(Info::parse(info_bytes)?);
                    // Now let's say I'm lazy to come up with anything else and
                    // assume that the 'announce' key always comes first.
                    break; // We're not interested in anything else.
                }
                _ => {
                    // Unknown field: skip the value
                    p.skip_any()?;
                }
            }
        }

        Ok(MetaInfoFile {
            announce: announce.ok_or(Error::UnknownField)?,
            info: info.ok_or(Error::UnknownField)?,
            info_hash,
        })
    }
}

impl<'a> Info<'a> {
    pub fn parse(input: &'a [u8]) -> Result<Self> {
        let mut p = BencodeParser::new(input);

        // Prepare default values (Option is useful here if fields are optional)
        let mut piece_length = None;
        let mut name = None;
        let mut pieces = None;
        let mut length = None;

        p.expect_dict_start()?;

        // Dictionary Loop
        while !p.match_dict_end() {
            let key = p.parse_str()?; // Keys are always strings

            match key {
                "pieces" => {
                    let piece_chunks = p.parse_str_bytes()?.as_chunks::<20>();
                    if !piece_chunks.1.is_empty() {
                        return Err(Error::InvalidSyntax);
                    }
                    pieces = Some(piece_chunks.0);
                }
                "length" => {
                    length = Some(p.parse_int()? as u32);
                }
                "piece length" => {
                    piece_length = Some(p.parse_int()? as u32);
                }
                "name" => {
                    name = Some(p.parse_str()?);
                }
                _ => {
                    // Unknown field: skip the value
                    p.skip_any()?;
                }
            }
        }

        // Validate we got everything
        Ok(Info {
            piece_length: piece_length.ok_or(Error::UnknownField)?,
            name: name.ok_or(Error::UnknownField)?,
            pieces: pieces.ok_or(Error::UnknownField)?,
            length: length.ok_or(Error::UnknownField)?,
        })
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    // Helper to create a 20-byte pseudo-hash for testing
    const HASH_A: [u8; 20] = [b'a'; 20];
    const HASH_B: [u8; 20] = [b'b'; 20];

    #[test]
    fn test_valid_torrent() {
        // Construct a valid bencoded dictionary.
        // Keys: announce, info (sorted alphabetically)
        let mut input = Vec::new();
        input.extend_from_slice(b"d");

        input.extend_from_slice(b"8:announce15:http://test.com");

        input.extend_from_slice(b"4:info");
        input.extend_from_slice(b"d");
        input.extend_from_slice(b"6:lengthi1048576e");
        input.extend_from_slice(b"4:name10:test.image");
        input.extend_from_slice(b"12:piece lengthi16384e");

        // Pieces: 40 bytes total (2 chunks of 20)
        input.extend_from_slice(b"6:pieces40:");
        input.extend_from_slice(&HASH_A);
        input.extend_from_slice(&HASH_B);

        input.extend_from_slice(b"e");
        input.extend_from_slice(b"i3ei9e"); // Extra junk fields after 'info'
        input.extend_from_slice(b"e");

        let torrent = MetaInfoFile::parse(&input).expect("Should parse valid input");

        assert_eq!(
            torrent.info_hash,
            sha1_smol::Sha1::from(&input[35..input.len() - 1 - 6]) // Exclude trailing junk
                .digest()
                .bytes()
        );

        assert_eq!(torrent.announce, "http://test.com");
        assert_eq!(torrent.info.length, 1048576);
        assert_eq!(torrent.info.name, "test.image");
        assert_eq!(torrent.info.piece_length, 16384);

        // Verify pieces array extraction
        assert_eq!(torrent.info.pieces.len(), 2);
        assert_eq!(torrent.info.pieces[0], HASH_A);
        assert_eq!(torrent.info.pieces[1], HASH_B);
    }

    #[test]
    fn test_invalid_pieces_length() {
        // Pieces length is 21 (valid bencode string, but invalid for logic because 21 % 20 != 0)
        let input = b"d6:lengthi1e4:name1:a12:piece lengthi1e6:pieces21:123456789012345678901e";

        let result = Info::parse(input);

        match result {
            Err(Error::InvalidSyntax) => (), // Pass
            _ => panic!("Should have failed due to remaining bytes in piece chunks"),
        }
    }

    #[test]
    fn test_skips_unknown_fields() {
        // Valid bencode with extra "junk" fields injected.
        // We inject "created by" and "comment" which are not in your struct.
        let mut input = Vec::new();
        input.extend_from_slice(b"d");

        input.extend_from_slice(b"7:comment15:this is ignored"); // Junk string
        input.extend_from_slice(b"6:lengthi100e");
        input.extend_from_slice(b"4:name3:log");
        input.extend_from_slice(b"12:piece lengthi16e");
        input.extend_from_slice(b"6:pieces20:");
        input.extend_from_slice(&HASH_A);
        input.extend_from_slice(b"10:some_stuffi999ee"); // Junk Integer inside dict

        input.extend_from_slice(b"e");

        let meta = Info::parse(&input).expect("Should successfully skip junk fields");

        assert_eq!(meta.name, "log");
        assert_eq!(meta.length, 100);
    }

    #[test]
    fn test_missing_mandatory_field() {
        // Missing "name" field
        let input = b"d6:lengthi100e12:piece lengthi16e6:pieces20:12345678901234567890e";

        let result = Info::parse(input);

        match result {
            Err(Error::UnknownField) => (), // Pass
            Ok(_) => panic!("Should fail because 'name' is missing"),
            Err(e) => panic!("Wrong error type: {:?}", e),
        }
    }

    #[test]
    fn test_empty_input_or_wrong_type() {
        // Input starts with 'i' (integer) instead of 'd' (dict)
        let result = Info::parse(b"i42e");
        assert!(matches!(result, Err(Error::ExpectedDict)));
    }
}
