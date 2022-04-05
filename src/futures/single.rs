use crate::driver::{Reactor, WakeState};
use crate::io::{EmmaFuture, _Poll};
use crate::Result;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

type PinnedEmmaFuture<'a, T> = Pin<Box<dyn EmmaFuture<Output = T> + Unpin + 'a>>;

pub struct Single<'single, T> {
    future: PinnedEmmaFuture<'single, T>,
    reactor: Reactor<'single>,
}

impl<'single, T: Unpin> Single<'single, T> {
    pub fn new(
        reactor: Reactor<'single>,
        future: PinnedEmmaFuture<'single, T>,
    ) -> Pin<Box<Single<'single, T>>> {
        Box::pin(Self { future, reactor })
    }
}

impl<T: Unpin> Future for Single<'_, T> {
    type Output = Result<T>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let reactor = Pin::new(&mut self.reactor);
        match reactor.wake() {
            Err(e) => return Poll::Ready(Err(e)),
            Ok(state) => match state {
                WakeState::Empty => unreachable!(),
                WakeState::Completion(tokens) => {
                    for token in tokens {
                        if self.future.as_ref().__token() == token {
                            match self.future.as_mut().__poll() {
                                _Poll::Ready(ret) => return Poll::Ready(Ok(ret)),
                                _Poll::Pending(_t) => {}
                            }
                        }
                    }
                }
            },
        }

        cx.waker().clone().wake();

        Poll::Pending
    }
}
