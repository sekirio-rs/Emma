//! JoinFuture

use crate::io::op::Ready;
use crate::io::EmmaFuture;
use crate::reactor::{Reactor, WakeState};
use crate::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

type JoinedFutures<'a> =
    HashMap<usize, Pin<Box<dyn EmmaFuture<Output = Result<Ready>> + Unpin + 'a>>>;
type JoinedReady = HashMap<usize, Result<Ready>>;
type PinnedEmmaFuture<'a> = Pin<Box<dyn EmmaFuture<Output = Result<Ready>> + Unpin + 'a>>;

pub struct Join<'emma> {
    futures: JoinedFutures<'emma>,
    reactor: Reactor<'emma>,
    result: JoinedReady,
}

impl<'emma> Join<'emma> {
    pub fn new(reactor: Reactor<'emma>) -> Pin<Box<Join<'emma>>> {
        Box::pin(Self {
            futures: HashMap::new(),
            reactor,
            result: HashMap::new(),
        })
    }

    pub fn join(mut self: Pin<&mut Self>, other: PinnedEmmaFuture<'emma>) -> Pin<&mut Self> {
        self.futures.insert(other.as_ref().__token(), other);
        self
    }
}

impl Future for Join<'_> {
    type Output = Result<JoinedReady>;
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
                                Poll::Ready(ret) => {
                                    self.result.insert(token, ret);
                                    self.futures.remove(&token); // task finished, drop
                                }
                                Poll::Pending => {}
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
