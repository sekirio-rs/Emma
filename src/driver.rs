//! Event Driver
use crate::{Emma, EmmaError, Handle, Inner as EmmaInner};
use crate::{EmmaState, Result};
use io_uring::IoUring;
use std::marker::PhantomData;
use std::pin::Pin;

pub struct Reactor<'emma> {
    uring_handle: Handle<IoUring>,
    inner_handle: Handle<EmmaInner>,
    _maker: PhantomData<&'emma Emma>,
}

impl Reactor<'_> {
    #[allow(clippy::needless_lifetimes)]
    pub fn new<'emma>(emma: &'emma Emma) -> Reactor<'emma> {
        Reactor {
            uring_handle: emma.uring.clone(),
            inner_handle: emma.inner.clone(),
            _maker: PhantomData,
        }
    }

    pub(crate) fn wake(self: Pin<&mut Self>) -> Result<WakeState> {
        // 1. submit_and_wait in uring
        // 2. traverse cqe, get related token
        // 3. change state in [`EmmaInner::slab`] via token

        let mut uring = self.uring_handle.borrow_mut();
        let mut inner = self.inner_handle.borrow_mut();

        if inner.slab.is_empty() {
            return Ok(WakeState::Empty);
        }

        // wait at least one submit completed by kernel
        if let Err(e) = uring.submit_and_wait(1) {
            return Err(EmmaError::IoError(e));
        }

        let mut cq = uring.completion();
        let mut wake_tokens = Vec::new();

        for cqe in &mut cq {
            let ret = cqe.result();
            let token = cqe.user_data() as usize;

            // todo
            if ret < 0 {
                return Err(EmmaError::IoError(std::io::Error::from_raw_os_error(-ret)));
            }

            unsafe {
                let state = inner.slab.get_unchecked_mut(token);

                match state {
                    EmmaState::Submitted | EmmaState::InExecution => {
                        wake_tokens.push(token);
                        *state = EmmaState::Completed(ret);
                    }
                    EmmaState::Completed(_) => unreachable!(),
                    EmmaState::_Reserved => unimplemented!(),
                }
            }
        }

        Ok(WakeState::Completion(wake_tokens))
    }
}

pub(crate) enum WakeState {
    Empty,
    Completion(Vec<usize>), // collection of token
}
