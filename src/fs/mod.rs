//! Implementation of asynchronous file system operation
//! Based on io_uring

use super::{Emma, EmmaState, Inner as EmmaInner};
use io_uring::{opcode, types};
use std::cell;
use std::fs::File as StdFile;
use std::future::Future;
use std::io as std_io;
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::ptr;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct File {
    std: Arc<StdFile>,
}

impl File {
    fn async_read<'emma>(
        &self,
        emma: &'emma mut Emma,
        buf: &'emma mut [u8],
    ) -> Pin<Box<EmmaRead<'emma>>> {
        // 1. push sqe to uring
        // 2. construct [`EmmaRead`]

        let uring = &mut emma.uring;
        let token = emma.inner.borrow_mut().slab.insert(EmmaState::Submitted);
        let entry = opcode::Read::new(
            types::Fd(self.std.as_raw_fd()),
            ptr::null_mut(),
            buf.len() as u32,
        )
        .build()
        .user_data(token as _);
        let mut sq = emma.uring.submission();

        unsafe {
            sq.push(&entry).unwrap();
        }

        sq.sync(); // sync to true uring

        Box::pin(EmmaRead {
            token,
            buf,
            handle: emma.inner.clone(),
        })
    }
}

pub struct EmmaRead<'emma> {
    token: usize,
    // buf reference, maybe unnecessary
    buf: &'emma [u8],
    // handle of Emma
    handle: Arc<cell::RefCell<EmmaInner>>,
}

impl<'emma> Future for EmmaRead<'emma> {
    type Output = std_io::Result<usize>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let handle = self.handle.as_ref();
        unsafe {
            match handle.borrow().slab.get_unchecked(self.token) {
                &EmmaState::Submitted => Poll::Pending,
                &EmmaState::Waiting => Poll::Pending,
                &EmmaState::Completed => Poll::Ready(Ok(self.buf.len())), // todo
            }
        }
    }
}
