use crate::io::{EmmaFuture, _Poll};
use std::pin::Pin;

pub struct Map<Fut, F> {
    future: Pin<Box<Fut>>,
    f: Option<F>,
}

impl<Fut, F> Map<Fut, F> {
    /// Creates a new Map
    pub(crate) fn new(future: Fut, f: F) -> Self {
        Self {
            future: Box::pin(future),
            f: Some(f),
        }
    }
}

impl<Fut, F, T> EmmaFuture for Map<Fut, F>
where
    Fut: EmmaFuture,
    F: FnOnce(Fut::Output) -> T + Unpin,
{
    type Output = T;

    fn __poll(mut self: Pin<&mut Self>) -> _Poll<Self::Output> {
        match self.future.as_mut().__poll() {
            _Poll::Ready(output) => unsafe {
                _Poll::Ready((self.f.take().unwrap_unchecked())(output))
            },
            _Poll::Pending(t) => _Poll::Pending(t),
        }
    }

    fn __token(self: Pin<&Self>) -> usize {
        self.future.as_ref().__token()
    }
}

pub trait IMap {
    type Fut: EmmaFuture;
    fn map<F>(self, f: F) -> Map<Self::Fut, F>;
}

impl<T: EmmaFuture> IMap for T {
    type Fut = T;
    fn map<F>(self, f: F) -> Map<Self::Fut, F> {
        Map::new(self, f)
    }
}
