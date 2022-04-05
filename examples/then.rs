use emma::futures::single::Single;
use emma::futures::then::IThen;
use emma::io::{EmmaFuture, _Poll};
use emma::{Emma, Reactor};
use std::future::Future;
use std::io;
use std::pin::Pin;

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let emma = emma::Builder::new().build().map_err(|e| e.as_io_error())?;

        let futa = FutA::new(1);

        let then_fut = futa.then(|token| FutB::new(token));

        let single = Single::new(Reactor::new(&emma), Box::pin(then_fut));

        single.await.map_err(|e| e.as_io_error())?;

        Ok(())
    })
}

struct FutA {
    token: usize,
}

impl FutA {
    pub fn new(token: usize) -> Self {
        Self { token }
    }
}

struct FutB {
    token: usize,
}

impl FutB {
    pub fn new(token: usize) -> Self {
        Self {
            token: token.wrapping_add(1),
        }
    }
}

impl EmmaFuture for FutA {
    type Output = usize;
    fn __poll(self: Pin<&mut Self>) -> _Poll<Self::Output> {
        println!("poll a: {}", self.token);

        _Poll::Ready(self.token)
    }
    fn __token(self: Pin<&Self>) -> usize {
        self.token
    }
}

impl EmmaFuture for FutB {
    type Output = usize;
    fn __poll(self: Pin<&mut Self>) -> _Poll<Self::Output> {
        println!("poll b: {}", self.token);

        _Poll::Ready(self.token)
    }
    fn __token(self: Pin<&Self>) -> usize {
        self.token
    }
}
