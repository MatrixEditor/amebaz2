use openssl::error::ErrorStack;
use std::io;

#[derive(Debug)]
pub enum Error {
    UnknownImageType(String),
    UnknownSectionType(String),
    InvalidEnumValue(String),
    IOError(io::Error),
    OpenSSLError(ErrorStack),
    UnsupportedHashAlgo(u8),
    NotImplemented(String),
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
