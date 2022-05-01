#![allow(non_snake_case)]

//! Copyright (C) 2022 SKTT1Ryze. All rights reserved.
//!
//! An asynchronous I/O library based on io_uring.
//!
//! `Emma` is a basic, high-performance asynchronous I/O library written in
//! rust-lang. At a high level, it provides a few major components:
//!
//! * APIs for [performing asynchronous filesystem operations][fs].
//! * APIs for [performing asynchronous network operations][net].
//!
//! [fs]: crate::fs
//! [net]: crate::net
//!
//! In Rust's asynchronous ecosystem, it needs something called asynchronous
//! runtime like [tokio] to execute and schedule asynchronous task. Emma can
//! work with any mainstream runtime including [tokio] and [async-std] and so
//! on.
//!
//! [tokio]: https://github.com/tokio-rs/tokio
//! [async-std]: https://github.com/async-rs/async-std
//!
//! It maybe a bit complex to write asynchronous code with Emma, so some helper
//! functions were provided in [alias] module. But **do not use them as much as
//! possible if you want to write low-cost code**. See API document for the
//! correct usage of Emma.
//!
//! [alias]: crate::alias
//!
//! ## Quick Start
//!
//! ```toml
//! emma = { git = "https://github.com/sekirio-rs/Emma" }
//! tokio = { version = "1", features = ["rt"]}
//! ```
//!
//! A tcp echo example:
//! ```Rust
//! use emma::{alias::*, net::tcp::listener::TcpListener};
//! use std::io;
//!
//! fn main() -> io::Result<()> {
//!     let rt = tokio::runtime::Builder::new_current_thread()
//!         .build()
//!         .unwrap();
//!
//!     rt.block_on(async {
//!         let emma = emma::Builder::new().build()?;
//!
//!         let listener = TcpListener::bind("127.0.0.1:3344")?;
//!
//!         let stream = accept_socket(&emma, &listener).await?;
//!
//!         let mut buf [0u8; 1024];
//!
//!         loop {
//!             let n = recv_msg(&emma, &mut buf, &stream).await?;
//!
//!             if n == 0 {
//!                 return Ok(());
//!             }
//!
//!             send_msg(&emma, &buf, &stream).await?;
//!         }
//!     })
//! }
//! ```

pub mod alias;
pub mod driver;
pub mod error;
pub mod fs;
pub mod futures;
pub mod io;
pub mod net;

use error::EmmaError;
use io_uring::IoUring;
use std::{cell, rc::Rc, result::Result as StdResult};

pub use driver::Reactor;
pub use futures::join::Join;

// RefCell for error message
// UnsafeCell for best performance
type Handle<T> = Rc<cell::RefCell<T>>;
pub type Result<T> = StdResult<T, error::EmmaError>;

/// Build [`Emma`] with custom configuration values.
///
/// # Examples
///
/// ```
/// use emma::Builder;
///
/// let emma = Builder::new().buid().unwrap();
/// ```
#[derive(Clone, Copy)]
pub struct Builder {
    /// Number of io_uring entries
    entries: u32,
}

impl Builder {
    const DEFAULT_ENTRIES: u32 = 1024; // must be power of 2

    /// Returns a new [`Emma`] builder initialized with default configuration
    /// values.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            entries: Self::DEFAULT_ENTRIES,
        }
    }

    /// Set the number of entries in io_uring instance.
    ///
    /// # Examples
    /// ```
    /// use emma::Builder;
    ///
    /// let emma = Builder::new().entries(1024).build().unwrap();
    /// ```
    ///
    /// # Safety
    ///
    /// Entries must be power of 2.
    pub fn entries(&mut self, entries: u32) -> &mut Self {
        self.entries = entries;
        self
    }

    /// Creats the configured [`Emma`]
    ///
    /// # Examples
    ///
    /// ```
    /// use emma::Builder;
    ///
    /// let emma = Builder::new().buid().unwrap();
    /// ```
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
    /// Single-thread shared handle of io_uring instance
    pub(crate) uring: Handle<IoUring>,
    /// Single-thread shared handle of [`Inner`]
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
