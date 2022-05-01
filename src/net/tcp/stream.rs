//! Copyright (C) 2022 SKTT1Ryze. All rights reserved.
use crate::{
    io::{op, recv, send, EmmaBuf},
    Emma, Result,
};
use std::{
    net,
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
    pin::Pin,
};

/// A TCP stream between a local and a remote socket.
///
/// # Examples
/// ```
/// todo!()
/// ```
pub struct TcpStream {
    // currently [`std::net::TcpStream`]
    inner: net::TcpStream,
}

impl TcpStream {
    /// Construct a [`TcpStream`] with given sockfd.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    ///
    /// # Safety
    /// TODO
    pub unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            inner: net::TcpStream::from_raw_fd(fd),
        }
    }

    /// Asynchorously recv messsage from currently [`TcpStream`].
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn recv<'emma, T: EmmaBuf + ?Sized>(
        &'emma self,
        emma: &'emma Emma,
        buf: &'emma mut T,
    ) -> Result<Pin<Box<op::Op<'emma, recv::Recv<'emma, T>>>>> {
        let fut = op::Op::async_recv(self.inner.as_raw_fd(), emma, buf)?;

        Ok(Box::pin(fut))
    }

    /// Asynchorously send messsage from currently [`TcpStream`].
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn send<'emma, T: EmmaBuf + Sync + ?Sized>(
        &'emma self,
        emma: &'emma Emma,
        buf: &'emma T,
    ) -> Result<Pin<Box<op::Op<'emma, send::Send_<'emma, T>>>>> {
        let fut = op::Op::async_send(self.inner.as_raw_fd(), emma, buf)?;

        Ok(Box::pin(fut))
    }
}
