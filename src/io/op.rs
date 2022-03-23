use crate::Emma;
use crate::EmmaState;
use crate::Handle;
use crate::Inner as EmmaInner;
use crate::Result;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

pub(crate) struct Op<'emma, T> {
    /// token in ['EmmaInner::slab']
    token: usize,
    /// handle of Emma
    handle: Handle<EmmaInner>,
    /// operation data
    data: Option<T>,
    /// make lifecycle
    _maker: PhantomData<&'emma EmmaInner>,
}

unsafe impl<T: Send> Send for Op<'_, T> {}

impl<'emma, T: Send> Op<'emma, T> {
    pub fn new(token: usize, emma: &'emma Emma, data: T) -> Op<'emma, T> {
        Op {
            token,
            handle: emma.inner.clone(),
            data: Some(data),
            _maker: PhantomData,
        }
    }
}

pub(crate) struct Ready {
    /// io_uring result
    pub(crate) uring_res: i32,
}

impl<T: Unpin> Future for Op<'_, T> {
    type Output = Result<Ready>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut handle = self.handle.as_ref().borrow_mut();
        let mut _ret: Option<i32> = None;

        unsafe {
            let state = handle.slab.get_unchecked_mut(self.token);

            match state {
                EmmaState::Submitted => {
                    *state = EmmaState::InExecution(cx.waker().clone());
                    return Poll::Pending;
                }
                EmmaState::InExecution(_waker) => unreachable!(),
                EmmaState::Completed(x) => {
                    _ret = Some(*x);
                }
                EmmaState::_Reserved => unimplemented!(),
            }

            if let Some(x) = _ret {
                let _ = handle.slab.remove(self.token);

                drop(handle);

                Poll::Ready(Ok(Ready { uring_res: x }))
            } else {
                Poll::Pending
            }
        }
    }
}
