use std::error::Error as StdError;
use std::fmt::Debug;

pub trait DebugError: StdError + Debug {}

impl<T> DebugError for T where T: StdError + Debug {}

pub enum EmmaError {
    IoError(std::io::Error),
    Other(Box<dyn DebugError + Send>),
}

impl Debug for EmmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmmaError::IoError(e) => e.fmt(f),
            EmmaError::Other(e) => e.fmt(f),
        }
    }
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
