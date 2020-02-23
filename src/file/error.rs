//! Error and result type for file transport

use self::Error::*;
use serde_json;
use std::io;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

/// An enum of all error kinds.
#[derive(Debug)]
pub enum Error {
    /// IO error
    Io(io::Error),
    /// JSON serialization error
    JsonSerialization(serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Io(ref err) => err.description(),
            JsonSerialization(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Io(ref err) => Some(&*err),
            JsonSerialization(ref err) => Some(&*err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonSerialization(err)
    }
}

/// SMTP result type
pub type FileResult = Result<(), Error>;
