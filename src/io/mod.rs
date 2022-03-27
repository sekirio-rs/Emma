//! Traits and something else for asynchronous I/O operations.
pub(crate) mod op;
pub(crate) mod open;
pub(crate) mod read;

use std::pin::Pin;
use std::task::Poll;

// todo: use macro
pub unsafe trait EmmaBuf: Unpin + 'static + Send {
    fn ptr(&self) -> *const u8;
    fn mut_ptr(&mut self) -> *mut u8;
    fn bytes(&self) -> usize;
}

unsafe impl EmmaBuf for Box<[u8]> {
    fn ptr(&self) -> *const u8 {
        self.as_ptr()
    }
    fn mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }
    fn bytes(&self) -> usize {
        self.len()
    }
}

unsafe impl<const N: usize> EmmaBuf for [u8; N] {
    fn ptr(&self) -> *const u8 {
        self.as_ptr()
    }
    fn mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }
    fn bytes(&self) -> usize {
        self.len()
    }
}

/// [`EmmaFuture`] for non-waker-poll design
pub trait EmmaFuture {
    type Output;
    fn __poll(self: Pin<&mut Self>) -> Poll<Self::Output>;
    fn __token(self: Pin<&Self>) -> usize;
}
