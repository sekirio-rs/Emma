use crate::Inner as EmmaInner;
use crate::Handle;
use crate::Result as EmmaResult;
use crate::EmmaState;
use crate::Emma;
use std::marker::PhantomData;
use std::future::Future;
use std::pin::Pin;
use std::task::{Poll, Context};


pub(crate) struct Op<'emma, T> {
    /// token in ['EmmaInner::slab']
    token: usize,
    /// handle of Emma
    handle: Handle<EmmaInner>,
    /// operation data
    data: Option<T>,
    /// make lifecycle
    _maker: PhantomData<&'emma EmmaInner>
}

impl<'emma, T> Op<'emma, T> {
    pub fn new(token: usize, emma: &'emma Emma, data: T) -> Op<'emma, T> {
        Op {
            token,
            handle: emma.inner.clone(),
            data: Some(data),
            _maker: PhantomData
        }
    }
}


pub(crate) struct Ready<T> {
    /// operation data
    pub(crate) data: T,
    /// io_uring result
    pub(crate) uring_res: i32
}

impl<T: Unpin> Future for Op<'_, T> {
    type Output = EmmaResult<Ready<T>>;

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
                EmmaState::_Reserved => unimplemented!()
            }

            if let Some(x) = _ret {
                let _ = handle.slab.remove(self.token);

                drop(handle);

                Poll::Ready(Ok(Ready {
                    data: self.get_mut().data.take().unwrap_unchecked(),
                    uring_res: x
                }))
            } else {
                Poll::Pending
            }
        }
        
    }
}
