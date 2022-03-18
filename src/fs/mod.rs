//! Implementation of asynchronous file system operation
//! Based on io_uring

use super::{Emma, EmmaState, Inner as EmmaInner};
use crate::io::EmmaBuf;
use io_uring::{opcode, types};
use std::cell;
use std::fs::File as StdFile;
use std::future::Future;
use std::io as std_io;
use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct File {
    std: Arc<StdFile>,
}

impl File {
    pub fn from_std(f: StdFile) -> Self {
        Self { std: Arc::new(f) }
    }

    pub fn as_std(&self) -> &StdFile {
        self.std.as_ref()
    }

    pub fn async_read<'emma, B: EmmaBuf>(
        &self,
        emma: Arc<cell::RefCell<Emma>>,
        buf: &'emma mut B,
    ) -> Pin<Box<EmmaRead<'emma>>> {
        // 1. push sqe to uring
        // 2. construct [`EmmaRead`]
        let mut emma = emma.borrow_mut();

        let token = emma.inner.borrow_mut().slab.insert(EmmaState::Submitted);
        let entry = opcode::Read::new(
            types::Fd(self.std.as_raw_fd()),
            buf.mut_ptr(),
            buf.bytes() as u32,
        )
        .build()
        .user_data(token as _);
        
        {

            let uring = &mut emma.uring;
            let mut sq = uring.submission();

            unsafe {
                sq.push(&entry).unwrap();
            }

            sq.sync(); // sync to true uring
        }
        
        let handle = emma.inner.clone();
        Box::pin(EmmaRead {
            token,
            handle,
            _marker: PhantomData,
        })
    }
}

pub struct EmmaRead<'emma> {
    // token in [`EmmaInner::slab`]
    token: usize,
    // handle of Emma
    handle: Arc<cell::RefCell<EmmaInner>>,
    // maker for lifecycle
    _marker: PhantomData<&'emma Box<dyn EmmaBuf>>,
}

impl<'emma> Future for EmmaRead<'emma> {
    type Output = std_io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut handle = self.handle.as_ref().borrow_mut();
        let mut _ret: Option<usize> = None;

        unsafe {
            let state = handle.slab.get_unchecked_mut(self.token);
            match state {
                EmmaState::Submitted => {
                    *state = EmmaState::InExecution(cx.waker().clone());
                    return Poll::Pending;
                }
                EmmaState::InExecution(_waker) => return Poll::Pending, // shouldn't reach here
                EmmaState::Completed(t) => {
                    _ret = Some(*t as usize);
                }
                EmmaState::_Reserved => unimplemented!(),
            }
        }

        if let Some(x) = _ret {
            let _ = handle.slab.remove(self.token);
            Poll::Ready(Ok(x))
        } else {
            Poll::Pending
        }
    }
}
