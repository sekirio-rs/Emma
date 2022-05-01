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

#[allow(clippy::missing_safety_doc, missing_docs)]
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

/// Emma version of [`std::task::Poll`]
pub enum _Poll<T> {
    /// Emma version of [`std::task::Poll::Ready`]
    Ready(T),
    /// Emma version of [`std::task::Poll::Pending`]
    Pending(Option<usize>),
}

/// [`EmmaFuture`] for non-waker-poll design
///
/// Emma version of [`std::future::Future`]
pub trait EmmaFuture {
    #[allow(missing_docs)]
    type Output;
    /// Emma verson of [`std::future::Future::poll`]
    fn __poll(self: Pin<&mut Self>) -> _Poll<Self::Output>;
    /// Return token for identify the [`EmmaFuture`]
    fn __token(self: Pin<&Self>) -> usize;
}
