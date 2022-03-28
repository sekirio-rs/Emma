use crate::io::{EmmaFuture, _Poll};
use std::pin::Pin;

pub enum Flatten<Fut1, Fut2> {
    First { fut: Pin<Box<Fut1>> },
    Second { fut: Pin<Box<Fut2>> },
    End,
}

impl<Fut1, Fut2> Flatten<Fut1, Fut2> {
    pub(crate) fn new(future: Fut1) -> Self {
        Self::First {
            fut: Box::pin(future),
        }
    }
}

impl<Fut> EmmaFuture for Flatten<Fut, Fut::Output>
where
    Fut: EmmaFuture,
    Fut::Output: EmmaFuture,
{
    type Output = <Fut::Output as EmmaFuture>::Output;

    fn __poll(mut self: Pin<&mut Self>) -> _Poll<Self::Output> {
        match &mut *self {
            Flatten::First { fut: ref mut fut1 } => match fut1.as_mut().__poll() {
                _Poll::Pending(t) => _Poll::Pending(t),
                _Poll::Ready(fut2) => {
                    let fut2 = Box::pin(fut2);
                    let token = fut2.as_ref().__token();

                    *self = Flatten::Second { fut: fut2 };
                    _Poll::Pending(Some(token))
                }
            },
            Flatten::Second { fut: ref mut fut2 } => match fut2.as_mut().__poll() {
                _Poll::Pending(t) => _Poll::Pending(t),
                _Poll::Ready(ret) => {
                    *self = Flatten::End;

                    _Poll::Ready(ret)
                }
            },
            Flatten::End => unreachable!(),
        }
    }

    fn __token(self: Pin<&Self>) -> usize {
        match &*self {
            Flatten::First { fut: fut1 } => fut1.as_ref().__token(),
            Flatten::Second { fut: fut2 } => fut2.as_ref().__token(),
            Flatten::End => unreachable!(),
        }
    }
}
