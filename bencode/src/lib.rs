#![cfg_attr(not(test), no_std)]

pub use crate::deserialize::BencodeParser;
use defmt::Format;

mod deserialize;

#[derive(Debug, Clone, Copy, Format)]
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
