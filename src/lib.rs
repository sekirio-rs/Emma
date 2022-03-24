//! Asynchronous I/O library based on io_uring.

#![allow(non_snake_case)]
pub mod error;
pub mod fs;
pub mod io;
pub mod join;
mod net;
pub mod reactor;
use error::EmmaError;
use io_uring::IoUring;
use std::cell;
use std::rc::Rc;
use std::result::Result as StdResult;
type Handle<T> = Rc<cell::RefCell<T>>;
type Result<T> = StdResult<T, error::EmmaError>;

/// Build [`Emma`] with custom configuration values.
pub struct Builder {
    /// Number of io_uring entries
    entries: u32,
    // todo
}

impl Builder {
    const DEFAULT_ENTRIES: u32 = 1024; // must be power of 2
    pub fn new() -> Self {
        Self {
            entries: Self::DEFAULT_ENTRIES,
        }
    }
    pub fn entries(&mut self, entries: u32) -> &mut Self {
        self.entries = entries;
        self
    }
    pub fn build(self) -> Result<Emma> {
        let uring = IoUring::new(self.entries).map_err(|e| EmmaError::IoError(e))?;
        let inner = Inner {
            slab: slab::Slab::with_capacity(Self::DEFAULT_ENTRIES as usize * 10),
        };
        Ok(Emma {
            uring: Rc::new(cell::RefCell::new(uring)),
            inner: Rc::new(cell::RefCell::new(inner)),
        })
    }
}

pub struct Emma {
    pub(crate) uring: Handle<IoUring>,
    pub(crate) inner: Handle<Inner>, // use UnsafeCell for best performance
}

struct Inner {
    pub(crate) slab: slab::Slab<EmmaState>,
}

pub(crate) enum EmmaState {
    Submitted,
    InExecution,
    Completed(i32),
    _Reserved,
}
