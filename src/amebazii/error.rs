use hex::FromHexError;
use openssl::error::ErrorStack;
use std::{io};
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    UnknownImageType(u8),
    UnknownSectionType(String),
    InvalidEnumValue(String),
    IOError(io::Error),
    OpenSSLError(ErrorStack),
    Utf8Error(FromUtf8Error),
    UnsupportedHashAlgo(u8),
    NotImplemented(String),
    InvalidState(String),
    SerdeJSONError(serde_json::Error),
    UnknownNVDMType(String),

    // REVISIT: must be reworked
    // individual parsing errors
    MalformedKeyblock(String),
    MalformedImageHeader(String),
    MalfromedPartTab(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}

impl From<ErrorStack> for Error {
    fn from(err: ErrorStack) -> Self {
        Error::OpenSSLError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJSONError(err)
    }
}

impl From<FromHexError> for Error {
    fn from(err: FromHexError) -> Self {
        Error::InvalidState(err.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::Utf8Error(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}