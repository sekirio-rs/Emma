//! Asynchronous I/O library based on io_uring.

#![allow(non_snake_case)]
pub mod fs;
mod io;
mod net;

use std::cell;
use std::io as std_io;
use std::sync::Arc;
use std::task::Waker;
use io_uring::IoUring;

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
    pub fn build(self) -> std_io::Result<Emma> {
        let uring = IoUring::new(self.entries)?;
        let inner = Inner {
            slab: slab::Slab::with_capacity(Self::DEFAULT_ENTRIES as usize * 10),
        };
        Ok(Emma {
            uring,
            inner: Arc::new(cell::RefCell::new(inner)),
        })
    }
}

// Send + !Sync
pub struct Emma {
    pub(crate) uring: IoUring,
    pub(crate) inner: Arc<cell::RefCell<Inner>>, // use UnsafeCell for best performance
}

struct Inner {
    pub(crate) slab: slab::Slab<EmmaState>,
}

pub(crate) enum EmmaState {
    Submitted,
    Waiting(Waker),
    Completed,
    Reserved
}
