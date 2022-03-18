//! Asynchronous I/O library based on io_uring.

#![allow(non_snake_case)]
pub mod fs;
mod io;
mod net;

use io_uring::IoUring;
use std::cell;
use std::future::Future;
use std::io as std_io;
use std::sync::Arc;
use std::task::Waker;

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
    InExecution(Waker),
    Completed(i32),
    _Reserved,
}

pub struct EmmaReactor(Arc<cell::RefCell<Emma>>);

impl EmmaReactor {
    pub fn from_emma(emma: &Arc<cell::RefCell<Emma>>) -> EmmaReactor {
        EmmaReactor(emma.clone())
    }
}

impl Future for EmmaReactor {
    type Output = std_io::Result<()>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // 1. check io_uring
        // 2. wake reference task
        // 3. wake or return

        let mut emma = self.get_mut().0.borrow_mut();
        let inner = emma.inner.clone();
        let uring = &mut emma.uring;

        if inner.borrow().slab.is_empty() {
            return std::task::Poll::Ready(Ok(()));
        }

        if let Err(e) = uring.submit_and_wait(1) {
            return std::task::Poll::Ready(Err(e));
        }

        let mut cq = uring.completion();

        for cqe in &mut cq {
            let ret = cqe.result();
            let token = cqe.user_data() as usize;

            if ret < 0 {
                return std::task::Poll::Ready(Err(std_io::Error::from_raw_os_error(-ret)));
            }

            unsafe {
                let mut inner = inner.borrow_mut();
                let state = inner.slab.get_unchecked_mut(token);

                if let EmmaState::InExecution(waker) = state {
                    waker.clone().wake();
                }

                *state = EmmaState::Completed(ret)
            }
        }

        cx.waker().clone().wake();
        return std::task::Poll::Pending;
    }
}
