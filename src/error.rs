use std::{error::Error as StdError, fmt::Debug};

pub trait DebugError: StdError + Debug {}

impl<T> DebugError for T where T: StdError + Debug {}

pub enum EmmaError {
    IoError(std::io::Error),
    Other(Box<dyn DebugError + Send>),
}

impl EmmaError {
    pub fn as_io_error(&self) -> std::io::Error {
        match self {
            EmmaError::IoError(e) => std::io::Error::from(e.kind()),
            EmmaError::Other(_e) => {
                std::io::Error::new(std::io::ErrorKind::Other, "io_uring error")
            }
        }
    }
}

impl Debug for EmmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmmaError::IoError(e) => e.fmt(f),
            EmmaError::Other(e) => e.fmt(f),
        }
    }
}

impl std::fmt::Display for EmmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn Debug).fmt(f)
    }
}

impl StdError for EmmaError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            EmmaError::IoError(e) => e.source(),
            EmmaError::Other(e) => e.source(),
        }
    }
}

impl From<EmmaError> for std::io::Error {
    fn from(e: EmmaError) -> Self {
        e.as_io_error()
    }
}
