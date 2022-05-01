//! Copyright (C) 2022 SKTT1Ryze. All rights reserved.
//! Traits and something else for asynchronous I/O operations.
pub(crate) mod accept;
pub(crate) mod close;
pub(crate) mod op;
pub(crate) mod open;
pub(crate) mod read;
pub(crate) mod recv;
pub(crate) mod send;
pub(crate) mod write;

use std::pin::Pin;

#[allow(clippy::missing_safety_doc)]
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

unsafe impl EmmaBuf for [u8] {
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

pub enum _Poll<T> {
    Ready(T),
    Pending(Option<usize>),
}

/// [`EmmaFuture`] for non-waker-poll design
pub trait EmmaFuture {
    type Output;
    fn __poll(self: Pin<&mut Self>) -> _Poll<Self::Output>;
    fn __token(self: Pin<&Self>) -> usize;
}

// impl<T> EmmaFuture for Pin<Box<T>>
// where
//     T: EmmaFuture,
// {
//     type Output = <T as EmmaFuture>::Output;
//
//     fn __poll(mut self: Pin<&mut Self>) -> _Poll<Self::Output> {
//         self.as_mut().__poll()
//     }
//
//     fn __token(self: Pin<&Self>) -> usize {
//         self.as_ref().__token()
//     }
// }

// impl<'a, T> EmmaFuture for Pin<Box<dyn EmmaFuture<Output = T> + 'a + Unpin>>
// {     type Output = T;
//
//     fn __poll(mut self: Pin<&mut Self>) -> _Poll<Self::Output> {
//         self.as_mut().__poll()
//     }
//
//     fn __token(self: Pin<&Self>) -> usize {
//         self.as_ref().__token()
//     }
// }
