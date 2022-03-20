//! Implementation of asynchronous file system operation
//! Based on io_uring

use super::{Emma, EmmaState, Inner as EmmaInner};
use crate::error::EmmaError;
use crate::io::EmmaBuf;
use crate::Handle;
use crate::Result;
use io_uring::{opcode, types};
use std::fs::File as StdFile;
use std::future::Future;
use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct File {
    // todo
    std: Arc<StdFile>,
}

impl File {
    pub fn from_std(f: StdFile) -> Self {
        Self { std: Arc::new(f) }
    }

    pub fn as_std(&self) -> &StdFile {
        self.std.as_ref()
    }

    pub fn async_read<'emma, T: EmmaBuf>(
        &self,
        emma: &'emma Emma,
        buf: &'emma mut T,
    ) -> Result<EmmaRead<'emma>> {
        // 1. push sqe to uring submission queue
        // 2. construct [`EmmaRead`]

        let token = emma.inner.borrow_mut().slab.insert(EmmaState::Submitted);
        let entry = opcode::Read::new(
            types::Fd(self.std.as_raw_fd()),
            buf.mut_ptr(),
            buf.bytes() as u32,
        )
        .build()
        .user_data(token as _);

        let mut uring = emma.uring.borrow_mut();
        let mut sq = uring.submission();

        unsafe {
            if let Err(e) = sq.push(&entry) {
                return Err(EmmaError::Other(Box::new(e)));
            }
        }

        sq.sync(); // sync to true uring submission queue

        Ok(EmmaRead {
            token,
            handle: emma.inner.clone(),
            _marker: PhantomData,
        })
    }
}

pub struct EmmaRead<'emma> {
    // token in [`EmmaInner::slab`]
    token: usize,
    // handle of Emma
    handle: Handle<EmmaInner>,
    // maker for lifecycle
    _marker: PhantomData<&'emma Box<dyn EmmaBuf>>,
}

impl<'emma> Future for EmmaRead<'emma> {
    type Output = Result<usize>;

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
                EmmaState::InExecution(_waker) => unreachable!(),
                EmmaState::Completed(t) => {
                    _ret = Some(*t as usize);
                }
                EmmaState::_Reserved => unimplemented!(),
            }
        }

        if let Some(x) = _ret {
            let _ = handle.slab.remove(self.token); // remove state in slab
            Poll::Ready(Ok(x))
        } else {
            Poll::Pending
        }
    }
}
