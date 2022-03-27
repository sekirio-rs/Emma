use crate::error::EmmaError;
use crate::io::EmmaFuture;
use crate::Emma;
use crate::EmmaState;
use crate::Handle;
use crate::Inner as EmmaInner;
use crate::Result;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::Poll;

pub struct Op<'emma, T> {
    /// token in ['EmmaInner::slab']
    token: usize,
    /// handle of Emma
    handle: Handle<EmmaInner>,
    /// operation data
    data: Option<T>,
    /// make lifecycle
    _maker: PhantomData<&'emma EmmaInner>,
}

impl<'emma, T: Send> Op<'emma, T> {
    pub(crate) fn new(token: usize, emma: &'emma Emma, data: T) -> Op<'emma, T> {
        Op {
            token,
            handle: emma.inner.clone(),
            data: Some(data),
            _maker: PhantomData,
        }
    }

    pub(crate) fn async_op(
        emma: &'emma Emma,
        func: impl FnOnce(usize) -> (io_uring::squeue::Entry, T),
    ) -> Result<Op<'emma, T>> {
        // 1. alloc token in [`EmmaInner::slab`]
        // 2. construct [`io_uring::squeue::Entry`] sqe
        // 3. submit to io_uring with sqe
        // 4. construct [`Op<'_, T>`] and return

        let token = emma.inner.borrow_mut().slab.insert(EmmaState::Submitted);

        let (entry, data) = func(token);

        let mut uring = emma.uring.borrow_mut();

        if uring.submission().is_full() {
            uring.submit().map_err(|e| EmmaError::IoError(e))?; // flush to kernel
        }

        let mut sq = uring.submission();

        unsafe {
            if let Err(e) = sq.push(&entry) {
                return Err(EmmaError::Other(Box::new(e)));
            }
        }

        sq.sync(); // sync to true uring submission queue

        Ok(Op::new(token, emma, data))
    }
}

pub struct Ready {
    /// io_uring result
    pub uring_res: i32,
}

impl<T: Unpin> EmmaFuture for Op<'_, T> {
    type Output = Result<Ready>;

    fn __poll(self: Pin<&mut Self>) -> Poll<Self::Output> {
        let mut handle = self.handle.as_ref().borrow_mut();
        let mut _ret: Option<i32> = None;

        unsafe {
            let state = handle.slab.get_unchecked_mut(self.token);

            match state {
                EmmaState::Submitted => {
                    *state = EmmaState::InExecution;
                    return Poll::Pending;
                }
                EmmaState::InExecution => unreachable!(),
                EmmaState::Completed(x) => {
                    _ret = Some(*x);
                }
                EmmaState::_Reserved => unimplemented!(),
            }
        }

        if let Some(x) = _ret {
            let _ = handle.slab.remove(self.token);

            Poll::Ready(Ok(Ready { uring_res: x }))
        } else {
            Poll::Pending
        }
    }

    fn __token(self: Pin<&Self>) -> usize {
        self.token
    }
}
