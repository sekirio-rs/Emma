//! JoinFuture

use crate::driver::{Reactor, WakeState};
use crate::io::{EmmaFuture, _Poll};
use crate::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

type JoinedFutures<'a, T> = HashMap<usize, Pin<Box<dyn EmmaFuture<Output = T> + Unpin + 'a>>>;
type JoinedReady<T> = HashMap<usize, T>;
type PinnedEmmaFuture<'a, T> = Pin<Box<dyn EmmaFuture<Output = T> + Unpin + 'a>>;

pub struct Join<'emma, T> {
    futures: JoinedFutures<'emma, T>,
    reactor: Reactor<'emma>,
    result: JoinedReady<T>,
}

impl<'emma, T: Unpin> Join<'emma, T> {
    pub fn new(reactor: Reactor<'emma>) -> Pin<Box<Join<'emma, T>>> {
        Box::pin(Self {
            futures: HashMap::new(),
            reactor,
            result: HashMap::new(),
        })
    }

    pub fn join(mut self: Pin<&mut Self>, other: PinnedEmmaFuture<'emma, T>) -> Pin<&mut Self> {
        self.futures.insert(other.as_ref().__token(), other);
        self
    }
}

impl<T: Unpin> Future for Join<'_, T> {
    type Output = Result<JoinedReady<T>>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let reactor = Pin::new(&mut self.reactor);
        match reactor.wake() {
            Err(e) => return Poll::Ready(Err(e)),
            Ok(state) => {
                match state {
                    WakeState::Empty => return Poll::Ready(Ok(std::mem::take(&mut self.result))),
                    WakeState::Completion(tokens) => {
                        for token in tokens {
                            let pinned_fut = self.futures.get_mut(&token).unwrap().as_mut();
                            match pinned_fut.__poll() {
                                _Poll::Ready(ret) => {
                                    self.result.insert(token, ret);
                                    self.futures.remove(&token); // task finished, drop
                                }
                                _Poll::Pending(t) => {
                                    if let Some(new_token) = t {
                                        let future = self.futures.remove(&token).unwrap();
                                        self.futures.insert(new_token, future);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        cx.waker().clone().wake();

        Poll::Pending
    }
}
