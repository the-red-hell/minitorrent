use bencode::{BencodeParser, Error, Result};

#[derive(Debug, PartialEq)]
pub struct MetaInfo<'a> {
    pub announce: &'a str,
    pub length: i64,
    pub name: &'a str,
}

impl<'a> MetaInfo<'a> {
    pub fn parse(input: &'a [u8]) -> Result<Self> {
        let mut p = BencodeParser::new(input);

        // Prepare default values (Option is useful here if fields are optional)
        let mut announce = None;
        let mut length = None;
        let mut name = None;

        p.expect_dict_start()?;

        // Dictionary Loop
        while !p.match_dict_end() {
            let key = p.parse_str()?; // Keys are always strings

            match key {
                "announce" => {
                    announce = Some(p.parse_str()?);
                }
                "length" => {
                    length = Some(p.parse_int()?);
                }
                "name" => {
                    name = Some(p.parse_str()?);
                }
                _ => {
                    // Unknown field: skip the value!
                    p.skip_any()?;
                }
            }
        }

        // Validate we got everything
        Ok(MetaInfo {
            announce: announce.ok_or(Error::UnknownField)?,
            length: length.ok_or(Error::UnknownField)?,
            name: name.ok_or(Error::UnknownField)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_metainfo() {
        let input = b"d8:announce15:http://test.com6:lengthi1234e4:name8:testfilee";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.announce, "http://test.com");
        assert_eq!(meta.length, 1234);
        assert_eq!(meta.name, "testfile");
    }

    #[test]
    fn test_parse_metainfo_different_order() {
        let input = b"d4:name8:testfile6:lengthi1234e8:announce15:http://test.come";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.announce, "http://test.com");
        assert_eq!(meta.length, 1234);
        assert_eq!(meta.name, "testfile");
    }

    #[test]
    fn test_parse_metainfo_with_extra_fields() {
        let input = b"d8:announce15:http://test.com7:comment12:Test torrent6:lengthi1234e4:name8:testfile12:created datei1234567890ee";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.announce, "http://test.com");
        assert_eq!(meta.length, 1234);
        assert_eq!(meta.name, "testfile");
    }

    #[test]
    fn test_parse_metainfo_with_nested_dict() {
        let input = b"d8:announce15:http://test.com4:infod6:lengthi1234e4:name8:testfilee6:lengthi1234e4:name8:testfilee";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.announce, "http://test.com");
        assert_eq!(meta.length, 1234);
        assert_eq!(meta.name, "testfile");
    }

    #[test]
    fn test_parse_metainfo_with_list_field() {
        let input =
            b"d8:announce15:http://test.com10:extra-listli1ei2ei3ee6:lengthi1234e4:name8:testfilee";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.announce, "http://test.com");
        assert_eq!(meta.length, 1234);
        assert_eq!(meta.name, "testfile");
    }

    #[test]
    fn test_parse_metainfo_missing_announce() {
        let input = b"d6:lengthi1234e4:name8:testfilee";
        let result = MetaInfo::parse(input);
        assert!(matches!(result, Err(Error::UnknownField)));
    }

    #[test]
    fn test_parse_metainfo_missing_length() {
        let input = b"d8:announce15:http://test.com4:name8:testfilee";
        let result = MetaInfo::parse(input);
        assert!(matches!(result, Err(Error::UnknownField)));
    }

    #[test]
    fn test_parse_metainfo_missing_name() {
        let input = b"d8:announce15:http://test.com6:lengthi1234ee";
        let result = MetaInfo::parse(input);
        assert!(matches!(result, Err(Error::UnknownField)));
    }

    #[test]
    fn test_parse_metainfo_empty_dict() {
        let input = b"de";
        let result = MetaInfo::parse(input);
        assert!(matches!(result, Err(Error::UnknownField)));
    }

    #[test]
    fn test_parse_metainfo_not_dict() {
        let input = b"i42e";
        let result = MetaInfo::parse(input);
        assert!(matches!(result, Err(Error::ExpectedDict)));
    }

    #[test]
    fn test_parse_metainfo_invalid_announce_type() {
        // When announce is an integer, parse_str will fail with ExpectedString
        // But since we're in a dict loop, we need the value to actually be an int
        let input = b"d8:announcei42e6:lengthi1234e4:name8:testfilee";
        let result = MetaInfo::parse(input);
        // This actually succeeds because i42e is treated as missing 'announce' and having an unknown key
        // The real error would be if parse_str is called expecting a string where there's an int
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_metainfo_invalid_length_type() {
        let input = b"d8:announce15:http://test.com6:length4:spam4:name8:testfilee";
        let result = MetaInfo::parse(input);
        assert!(matches!(result, Err(Error::ExpectedInteger)));
    }

    #[test]
    fn test_parse_metainfo_invalid_name_type() {
        let input = b"d8:announce15:http://test.com6:lengthi1234e4:namei42ee";
        let result = MetaInfo::parse(input);
        assert!(matches!(result, Err(Error::ExpectedString)));
    }

    #[test]
    fn test_parse_metainfo_zero_length() {
        let input = b"d8:announce15:http://test.com6:lengthi0e4:name8:testfilee";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.length, 0);
    }

    #[test]
    fn test_parse_metainfo_negative_length() {
        let input = b"d8:announce15:http://test.com6:lengthi-100e4:name8:testfilee";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.length, -100);
    }

    #[test]
    fn test_parse_metainfo_large_length() {
        let input = b"d8:announce15:http://test.com6:lengthi9999999999e4:name8:testfilee";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.length, 9999999999);
    }

    #[test]
    fn test_parse_metainfo_empty_strings() {
        let input = b"d8:announce0:6:lengthi1234e4:name0:e";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.announce, "");
        assert_eq!(meta.name, "");
    }

    #[test]
    fn test_parse_metainfo_special_chars_in_strings() {
        let input =
            b"d8:announce23:http://test.com:8080/tr6:lengthi1234e4:name18:test file-v1.0.txte";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.announce, "http://test.com:8080/tr");
        assert_eq!(meta.name, "test file-v1.0.txt");
    }

    #[test]
    fn test_parse_metainfo_malformed_dict() {
        let input = b"d8:announce15:http://test.com6:lengthi1234e4:name8:testfile";
        let result = MetaInfo::parse(input);
        // Missing closing 'e', so it will succeed in parsing but fail to find all fields
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_metainfo_duplicate_keys() {
        // Last value wins
        let input = b"d8:announce10:http://old6:lengthi100e8:announce15:http://test.com6:lengthi1234e4:name8:testfilee";
        let meta = MetaInfo::parse(input).unwrap();

        assert_eq!(meta.announce, "http://test.com");
        assert_eq!(meta.length, 1234);
    }

    #[test]
    fn test_parse_metainfo_incomplete_key_value() {
        let input = b"d8:announcee";
        let result = MetaInfo::parse(input);
        assert!(result.is_err());
    }
}
