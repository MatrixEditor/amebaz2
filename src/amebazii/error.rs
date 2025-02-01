use hex::FromHexError;
use openssl::error::ErrorStack;
use std::{error, io};

#[derive(Debug)]
pub enum Error {
    UnknownImageType(u8),
    UnknownSectionType(String),
    InvalidEnumValue(String),
    IOError(io::Error),
    OpenSSLError(ErrorStack),
    UnsupportedHashAlgo(u8),
    NotImplemented(String),
    InvalidState(String),
    SerdeJSONError(serde_json::Error),

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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}