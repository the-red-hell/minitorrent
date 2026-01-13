#![no_std]
use core::str;

// 1. Simple Error Types
#[derive(Debug, Clone, Copy)]
pub enum Error {
    UnexpectedEof,
    InvalidSyntax,
    InvalidUtf8,
    ExpectedInteger,
    ExpectedString,
    ExpectedDict,
    UnknownField,
}

pub type Result<T> = core::result::Result<T, Error>;

// 2. The Parser Cursor
// Holds the current position in the byte slice.
pub struct BencodeParser<'a> {
    input: &'a [u8],
}

impl<'a> BencodeParser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input }
    }

    /// Peek at the next byte without consuming it
    pub fn peek(&self) -> Option<u8> {
        self.input.first().copied()
    }

    /// Consume the 'i'.. 'e' integer format
    pub fn parse_int(&mut self) -> Result<i64> {
        if self.peek() != Some(b'i') {
            return Err(Error::ExpectedInteger);
        }
        self.input = &self.input[1..]; // skip 'i'

        // Find position of 'e'
        let end = self.input.iter().position(|&b| b == b'e')
            .ok_or(Error::InvalidSyntax)?;

        let (int_bytes, rest) = self.input.split_at(end);
        
        // Bencode spec: integers cannot have leading plus sign
        if int_bytes.first() == Some(&b'+') {
            return Err(Error::InvalidSyntax);
        }
        
        // Parse the number
        let s = str::from_utf8(int_bytes).map_err(|_| Error::InvalidSyntax)?;
        let val = s.parse::<i64>().map_err(|_| Error::InvalidSyntax)?;

        self.input = &rest[1..]; // skip 'e'
        Ok(val)
    }

    /// Consume a length-prefixed string: "4:spam" -> "spam"
    pub fn parse_str(&mut self) -> Result<&'a str> {
        // Find the colon
        let colon_idx = self.input.iter().position(|&b| b == b':')
            .ok_or(Error::ExpectedString)?;

        let (len_bytes, rest) = self.input.split_at(colon_idx);
        
        // Parse length
        let len_str = str::from_utf8(len_bytes).map_err(|_| Error::InvalidSyntax)?;
        let len = len_str.parse::<usize>().map_err(|_| Error::InvalidSyntax)?;

        let rest = &rest[1..]; // skip ':'

        if rest.len() < len {
            return Err(Error::UnexpectedEof);
        }

        // Slice the string (Zero Copy!)
        let (s_bytes, remaining) = rest.split_at(len);
        let s = str::from_utf8(s_bytes).map_err(|_| Error::InvalidUtf8)?;

        self.input = remaining;
        Ok(s)
    }

    /// Crucial: Skips the next element (int, string, list, or dict)
    /// Needed when the input has fields your struct doesn't need.
    pub fn skip_any(&mut self) -> Result<()> {
        match self.peek() {
            Some(b'i') => { self.parse_int().map(|_| ()) }
            Some(b'0'..=b'9') => { self.parse_str().map(|_| ()) }
            Some(b'l') => {
                self.input = &self.input[1..]; // skip 'l'
                while self.peek() != Some(b'e') {
                    self.skip_any()?;
                }
                self.input = &self.input[1..]; // skip 'e'
                Ok(())
            }
            Some(b'd') => {
                self.input = &self.input[1..]; // skip 'd'
                while self.peek() != Some(b'e') {
                    self.skip_any()?; // key
                    self.skip_any()?; // value
                }
                self.input = &self.input[1..]; // skip 'e'
                Ok(())
            }
            _ => Err(Error::InvalidSyntax),
        }
    }

    /// Helper to start a dict
    pub fn expect_dict_start(&mut self) -> Result<()> {
        if self.peek() == Some(b'd') {
            self.input = &self.input[1..];
            Ok(())
        } else {
            Err(Error::ExpectedDict)
        }
    }
    
    /// Helper to end a dict
    pub fn match_dict_end(&mut self) -> bool {
        if self.peek() == Some(b'e') {
            self.input = &self.input[1..];
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int_positive() {
        let mut parser = BencodeParser::new(b"i42e");
        assert_eq!(parser.parse_int().unwrap(), 42);
    }

    #[test]
    fn test_parse_int_negative() {
        let mut parser = BencodeParser::new(b"i-42e");
        assert_eq!(parser.parse_int().unwrap(), -42);
    }

    #[test]
    fn test_parse_int_zero() {
        let mut parser = BencodeParser::new(b"i0e");
        assert_eq!(parser.parse_int().unwrap(), 0);
    }

    #[test]
    fn test_parse_int_large() {
        let mut parser = BencodeParser::new(b"i9223372036854775807e");
        assert_eq!(parser.parse_int().unwrap(), 9223372036854775807);
    }

    #[test]
    fn test_parse_int_missing_start() {
        let mut parser = BencodeParser::new(b"42e");
        assert!(matches!(parser.parse_int(), Err(Error::ExpectedInteger)));
    }

    #[test]
    fn test_parse_int_missing_end() {
        let mut parser = BencodeParser::new(b"i42");
        assert!(matches!(parser.parse_int(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_int_invalid_number() {
        let mut parser = BencodeParser::new(b"iabce");
        assert!(matches!(parser.parse_int(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_str_simple() {
        let mut parser = BencodeParser::new(b"4:spam");
        assert_eq!(parser.parse_str().unwrap(), "spam");
    }

    #[test]
    fn test_parse_str_empty() {
        let mut parser = BencodeParser::new(b"0:");
        assert_eq!(parser.parse_str().unwrap(), "");
    }

    #[test]
    fn test_parse_str_with_special_chars() {
        let mut parser = BencodeParser::new(b"11:hello world");
        assert_eq!(parser.parse_str().unwrap(), "hello world");
    }

    #[test]
    fn test_parse_str_missing_colon() {
        let mut parser = BencodeParser::new(b"4spam");
        assert!(matches!(parser.parse_str(), Err(Error::ExpectedString)));
    }

    #[test]
    fn test_parse_str_length_too_long() {
        let mut parser = BencodeParser::new(b"10:spam");
        assert!(matches!(parser.parse_str(), Err(Error::UnexpectedEof)));
    }

    #[test]
    fn test_parse_str_invalid_length() {
        let mut parser = BencodeParser::new(b"abc:spam");
        assert!(matches!(parser.parse_str(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_str_sequence() {
        let mut parser = BencodeParser::new(b"4:spam3:egg");
        assert_eq!(parser.parse_str().unwrap(), "spam");
        assert_eq!(parser.parse_str().unwrap(), "egg");
    }

    #[test]
    fn test_parse_int_sequence() {
        let mut parser = BencodeParser::new(b"i42ei-10e");
        assert_eq!(parser.parse_int().unwrap(), 42);
        assert_eq!(parser.parse_int().unwrap(), -10);
    }

    #[test]
    fn test_peek() {
        let parser = BencodeParser::new(b"i42e");
        assert_eq!(parser.peek(), Some(b'i'));
    }

    #[test]
    fn test_peek_empty() {
        let parser = BencodeParser::new(b"");
        assert_eq!(parser.peek(), None);
    }

    #[test]
    fn test_skip_any_integer() {
        let mut parser = BencodeParser::new(b"i42e4:spam");
        parser.skip_any().unwrap();
        assert_eq!(parser.parse_str().unwrap(), "spam");
    }

    #[test]
    fn test_skip_any_string() {
        let mut parser = BencodeParser::new(b"4:spami42e");
        parser.skip_any().unwrap();
        assert_eq!(parser.parse_int().unwrap(), 42);
    }

    #[test]
    fn test_skip_any_list() {
        let mut parser = BencodeParser::new(b"li42e4:spamei99e");
        parser.skip_any().unwrap();
        assert_eq!(parser.parse_int().unwrap(), 99);
    }

    #[test]
    fn test_skip_any_dict() {
        let mut parser = BencodeParser::new(b"d3:key5:valueei123e");
        parser.skip_any().unwrap();
        assert_eq!(parser.parse_int().unwrap(), 123);
    }

    #[test]
    fn test_skip_any_nested_list() {
        let mut parser = BencodeParser::new(b"lli42eeei99e");
        parser.skip_any().unwrap();
        assert_eq!(parser.parse_int().unwrap(), 99);
    }

    #[test]
    fn test_skip_any_nested_dict() {
        let mut parser = BencodeParser::new(b"d3:keyd6:nestedi1eeei42e");
        parser.skip_any().unwrap();
        assert_eq!(parser.parse_int().unwrap(), 42);
    }

    #[test]
    fn test_expect_dict_start() {
        let mut parser = BencodeParser::new(b"d3:key5:valuee");
        assert!(parser.expect_dict_start().is_ok());
        assert_eq!(parser.parse_str().unwrap(), "key");
    }

    #[test]
    fn test_expect_dict_start_fail() {
        let mut parser = BencodeParser::new(b"i42e");
        assert!(matches!(parser.expect_dict_start(), Err(Error::ExpectedDict)));
    }

    #[test]
    fn test_match_dict_end() {
        let mut parser = BencodeParser::new(b"e");
        assert!(parser.match_dict_end());
    }

    #[test]
    fn test_match_dict_end_fail() {
        let mut parser = BencodeParser::new(b"i42e");
        assert!(!parser.match_dict_end());
    }

    #[test]
    fn test_dict_parsing_workflow() {
        let mut parser = BencodeParser::new(b"d3:key5:value3:fooi42ee");
        parser.expect_dict_start().unwrap();
        
        // Parse first key-value pair
        assert_eq!(parser.parse_str().unwrap(), "key");
        assert_eq!(parser.parse_str().unwrap(), "value");
        
        // Parse second key-value pair
        assert_eq!(parser.parse_str().unwrap(), "foo");
        assert_eq!(parser.parse_int().unwrap(), 42);
        
        // Check dict end
        assert!(parser.match_dict_end());
    }

    #[test]
    fn test_skip_unknown_fields_in_dict() {
        let mut parser = BencodeParser::new(b"d7:ignored4:datai99e");
        parser.expect_dict_start().unwrap();
        
        // Skip first key-value pair
        parser.skip_any().unwrap(); // key
        parser.skip_any().unwrap(); // value
        
        // Parse the integer that follows
        assert_eq!(parser.parse_int().unwrap(), 99);
    }

    #[test]
    fn test_empty_dict() {
        let mut parser = BencodeParser::new(b"de");
        parser.expect_dict_start().unwrap();
        assert!(parser.match_dict_end());
    }

    #[test]
    fn test_empty_list_skip() {
        let mut parser = BencodeParser::new(b"lei42e");
        parser.skip_any().unwrap();
        assert_eq!(parser.parse_int().unwrap(), 42);
    }

    // Error case tests
    #[test]
    fn test_skip_any_invalid_start() {
        let mut parser = BencodeParser::new(b"x");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_skip_any_empty_input() {
        let mut parser = BencodeParser::new(b"");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_skip_any_list_missing_end() {
        let mut parser = BencodeParser::new(b"li42e");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_skip_any_dict_missing_end() {
        let mut parser = BencodeParser::new(b"d3:key5:value");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_skip_any_dict_odd_elements() {
        let mut parser = BencodeParser::new(b"d3:keye");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_str_invalid_utf8() {
        // Invalid UTF-8 sequence
        let mut parser = BencodeParser::new(b"4:\xff\xfe\xfd\xfc");
        assert!(matches!(parser.parse_str(), Err(Error::InvalidUtf8)));
    }

    #[test]
    fn test_parse_int_overflow() {
        // Number larger than i64::MAX
        let mut parser = BencodeParser::new(b"i99999999999999999999e");
        assert!(matches!(parser.parse_int(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_int_empty() {
        let mut parser = BencodeParser::new(b"ie");
        assert!(matches!(parser.parse_int(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_int_invalid_chars() {
        let mut parser = BencodeParser::new(b"i42xe");
        assert!(matches!(parser.parse_int(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_int_multiple_signs() {
        let mut parser = BencodeParser::new(b"i--42e");
        assert!(matches!(parser.parse_int(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_int_plus_sign() {
        let mut parser = BencodeParser::new(b"i+42e");
        assert!(matches!(parser.parse_int(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_str_negative_length() {
        let mut parser = BencodeParser::new(b"-5:hello");
        assert!(matches!(parser.parse_str(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_str_zero_length() {
        let mut parser = BencodeParser::new(b"0:");
        assert_eq!(parser.parse_str().unwrap(), "");
    }

    #[test]
    fn test_skip_any_list_with_invalid_element() {
        let mut parser = BencodeParser::new(b"lxe");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_skip_any_dict_with_invalid_key() {
        let mut parser = BencodeParser::new(b"dxi42ee");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_skip_any_dict_with_invalid_value() {
        let mut parser = BencodeParser::new(b"d3:keyxe");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_expect_dict_start_on_list() {
        let mut parser = BencodeParser::new(b"li42ee");
        assert!(matches!(parser.expect_dict_start(), Err(Error::ExpectedDict)));
    }

    #[test]
    fn test_expect_dict_start_on_string() {
        let mut parser = BencodeParser::new(b"4:spam");
        assert!(matches!(parser.expect_dict_start(), Err(Error::ExpectedDict)));
    }

    #[test]
    fn test_expect_dict_start_empty_input() {
        let mut parser = BencodeParser::new(b"");
        assert!(matches!(parser.expect_dict_start(), Err(Error::ExpectedDict)));
    }

    #[test]
    fn test_parse_int_eof_in_number() {
        let mut parser = BencodeParser::new(b"i42");
        assert!(matches!(parser.parse_int(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_str_eof_in_data() {
        let mut parser = BencodeParser::new(b"10:short");
        assert!(matches!(parser.parse_str(), Err(Error::UnexpectedEof)));
    }

    #[test]
    fn test_nested_structure_errors() {
        let mut parser = BencodeParser::new(b"lli42e");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_dict_nested_error() {
        let mut parser = BencodeParser::new(b"d3:keyd3:foo");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
    }

    #[test]
    fn test_parse_str_max_length() {
        let mut parser = BencodeParser::new(b"3:abc");
        assert_eq!(parser.parse_str().unwrap(), "abc");
    }

    #[test]
    fn test_multiple_consecutive_errors() {
        let mut parser = BencodeParser::new(b"xyz");
        assert!(matches!(parser.skip_any(), Err(Error::InvalidSyntax)));
        // Parser state after error - should still be at 'x'
        assert_eq!(parser.peek(), Some(b'x'));
    }
}
