use git2::Error as Git2Error;
use serde_json::Error as SerdeJsonError;

use std::error::Error as StdError;
use std::fmt::Error as FmtError;
use std::fmt::{self, Display};
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Fmt(FmtError),
    SerdeJson(SerdeJsonError),
    Git2(Git2Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => Display::fmt(&err, f),
            Error::Fmt(err) => Display::fmt(&err, f),
            Error::SerdeJson(err) => Display::fmt(&err, f),
            Error::Git2(err) => Display::fmt(&err, f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Fmt(err) => Some(err),
            Error::Git2(err) => Some(err),
            Error::SerdeJson(err) => Some(err),
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

impl From<FmtError> for Error {
    fn from(err: FmtError) -> Self {
        Self::Fmt(err)
    }
}

impl From<SerdeJsonError> for Error {
    fn from(err: SerdeJsonError) -> Self {
        Self::SerdeJson(err)
    }
}

impl From<Git2Error> for Error {
    fn from(err: Git2Error) -> Self {
        Self::Git2(err)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
