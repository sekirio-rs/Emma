//! Asynchronous I/O library based on io_uring.

#![allow(non_snake_case)]
pub mod alias;
pub mod driver;
pub mod error;
pub mod fs;
pub mod futures;
pub mod io;
pub mod net;

use error::EmmaError;
use io_uring::IoUring;
use std::cell;
use std::rc::Rc;
use std::result::Result as StdResult;

pub use driver::Reactor;
pub use futures::join::Join;

// RefCell for error message
// UnsafeCell for best performance
type Handle<T> = Rc<cell::RefCell<T>>;
pub type Result<T> = StdResult<T, error::EmmaError>;

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
        let uring = IoUring::new(self.entries).map_err(EmmaError::IoError)?;
        let inner = Inner {
            slab: slab::Slab::with_capacity(Self::DEFAULT_ENTRIES as usize * 10),
        };
        Ok(Emma {
            uring: Rc::new(cell::RefCell::new(uring)),
            inner: Rc::new(cell::RefCell::new(inner)),
        })
    }
}

/// Structure which holds the io_uring instance
/// and states of submission provided by user.
///
/// Designed as Send + !Sync
pub struct Emma {
    pub(crate) uring: Handle<IoUring>,
    pub(crate) inner: Handle<Inner>,
}

impl Clone for Emma {
    fn clone(&self) -> Self {
        Self {
            uring: self.uring.clone(),
            inner: self.inner.clone(),
        }
    }
}

/// Inner for [`Emma`], and will be shared with [`Op`]
/// in the same thread.
struct Inner {
    pub(crate) slab: slab::Slab<EmmaState>,
}

pub(crate) enum EmmaState {
    /// Operation has been submitted to io_uring
    Submitted,
    /// Waiting for kernel to complete the submission
    InExecution,
    /// Submission has been completed
    Completed(i32),
    _Reserved,
}
