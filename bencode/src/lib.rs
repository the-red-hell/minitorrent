#![cfg_attr(not(test), no_std)]

use core::str::Utf8Error;

pub use crate::deserialize::BencodeParser;

mod deserialize;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "log", derive(Debug))]
pub enum Error {
    UnexpectedEof,
    InvalidSyntax,
    InvalidUtf8(Utf8Error),
    ExpectedInteger,
    ExpectedString,
    ExpectedDict,
    UnknownField,
}

#[cfg(feature = "defmt")]
impl defmt::Format for Error {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Error::InvalidUtf8(e) => {
                defmt::write!(
                    f,
                    "Invalid UTF-8: valid_up_to: {}, error_len: {:?}",
                    e.valid_up_to(),
                    e.error_len()
                )
            }
            kind => defmt::write!(f, "{:?}", kind),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
