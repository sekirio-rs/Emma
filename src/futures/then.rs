use super::flatten::Flatten;
use super::map::Map;
use crate::io::EmmaFuture;
use crate::io::_Poll;
use std::pin::Pin;

pub struct Then<Fut1, Fut2, F>(Flatten<Map<Fut1, F>, Fut2>);

impl<Fut1, Fut2, F> Then<Fut1, Fut2, F> {
    pub fn new(future: Fut1, f: F) -> Self {
        Self(Flatten::new(Map::new(future, f)))
    }
}

impl<Fut, F, T> EmmaFuture for Then<Fut, T, F>
where
    Fut: EmmaFuture,
    F: FnOnce(Fut::Output) -> T + Unpin,
    T: EmmaFuture,
{
    type Output = <T as EmmaFuture>::Output;

    fn __poll(mut self: Pin<&mut Self>) -> _Poll<Self::Output> {
        Pin::new(&mut self.0).__poll()
    }

    fn __token(self: Pin<&Self>) -> usize {
        Pin::new(&self.0).__token()
    }
}

pub trait IThen {
    type Fut: EmmaFuture;
    fn then<F, T>(self, f: F) -> Then<Self::Fut, T, F>
    where
        Self: Sized;
}

impl<H: EmmaFuture> IThen for H {
    type Fut = H;
    fn then<F, T>(self, f: F) -> Then<Self::Fut, T, F>
    where
        Self: Sized,
    {
        Then::new(self, f)
    }
}
