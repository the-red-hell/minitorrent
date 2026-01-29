#![cfg_attr(not(test), no_std)]

use core::str::Utf8Error;

pub use crate::deserialize::BencodeParser;

mod deserialize;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    UnexpectedEof,
    InvalidSyntax,
    InvalidUtf8(Utf8Error),
    ExpectedInteger,
    ExpectedString,
    ExpectedDict,
    UnknownField,
}

pub type Result<T> = core::result::Result<T, Error>;
