//! Asynchronous I/O library based on io_uring.

#![allow(non_snake_case)]
pub mod error;
pub mod fs;
mod io;
mod net;

use error::EmmaError;
use io_uring::IoUring;
use std::cell;
use std::future::Future;
use std::marker::PhantomData;
use std::result::Result as StdResult;
use std::sync::Arc;
use std::task::Waker;

type Handle<T> = Arc<cell::RefCell<T>>;
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
            uring: Arc::new(cell::RefCell::new(uring)),
            inner: Arc::new(cell::RefCell::new(inner)),
        })
    }
}

// Send + !Sync
pub struct Emma {
    pub(crate) uring: Handle<IoUring>,
    pub(crate) inner: Handle<Inner>, // use UnsafeCell for best performance
}

struct Inner {
    pub(crate) slab: slab::Slab<EmmaState>,
}

pub(crate) enum EmmaState {
    Submitted,
    InExecution(Waker),
    Completed(i32),
    _Reserved,
}

pub struct EmmaReactor<'emma> {
    uring_handle: Handle<IoUring>,
    inner_handle: Handle<Inner>,
    _marker: PhantomData<&'emma Emma>,
}

impl EmmaReactor<'_> {
    pub fn from_emma<'emma>(emma: &'emma Emma) -> EmmaReactor<'emma> {
        EmmaReactor {
            uring_handle: emma.uring.clone(),
            inner_handle: emma.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl Future for EmmaReactor<'_> {
    type Output = Result<()>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // 1. check uring instance
        // 2. traverse cqe, and wake related task
        // 3. wake itself or return

        let mut uring = self.uring_handle.borrow_mut();
        let mut inner = self.inner_handle.borrow_mut();

        if inner.slab.is_empty() {
            // all tasks have been completed, return
            return std::task::Poll::Ready(Ok(()));
        }

        // wait at least one submit completed by kernel
        if let Err(e) = uring.submit_and_wait(1) {
            return std::task::Poll::Ready(Err(EmmaError::IoError(e)));
        }

        let mut cq = uring.completion();

        for cqe in &mut cq {
            let ret = cqe.result();
            let token = cqe.user_data() as usize;

            if ret < 0 {
                return std::task::Poll::Ready(Err(EmmaError::IoError(
                    std::io::Error::from_raw_os_error(-ret),
                )));
            }

            unsafe {
                let state = inner.slab.get_unchecked_mut(token);

                if let EmmaState::InExecution(waker) = state {
                    // wake related task
                    waker.clone().wake();
                }

                *state = EmmaState::Completed(ret)
            }
        }

        cq.sync(); // sync to true completion queue

        // wake reactor itself
        cx.waker().clone().wake();

        return std::task::Poll::Pending;
    }
}
