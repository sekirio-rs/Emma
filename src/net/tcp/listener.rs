//! Copyright (C) 2022 SKTT1Ryze. All rights reserved.
use super::{super::unix, stream::TcpStream};
use crate::{
    futures::map::Map,
    io::{
        op::{self, Ready},
        EmmaFuture,
    },
    Emma, EmmaError, Result,
};
use std::{
    io, net,
    os::unix::io::{AsRawFd, FromRawFd},
    pin::Pin,
};

const BACKLOG: u32 = 1024;

/// A TCP socket server, listening for connections.
///
/// You can accept a new connection asynchorously by calling the
/// [`accept`](`TcpListener::accept`) method.
///
/// # Examples
/// ```
/// todo!()
/// ```
pub struct TcpListener {
    inner: net::TcpListener,
}

impl TcpListener {
    /// Creates a new TcpListener, witch will be bound to the specified address.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn bind<A: net::ToSocketAddrs>(addr: A) -> Result<Self> {
        let addr = addr
            .to_socket_addrs()
            .map_err(EmmaError::IoError)?
            .next()
            .ok_or_else(|| {
                EmmaError::IoError(io::Error::new(io::ErrorKind::Other, "invalid ip address"))
            })?;

        let socket = unix::new_socket(addr)?;

        unix::set_reuseaddr(socket, true)?;

        unix::set_reuseport(socket, true)?;

        unix::bind(socket, addr)?;

        unix::listen(socket, BACKLOG)?;

        let listener = unsafe { net::TcpListener::from_raw_fd(socket) };

        Ok(Self { inner: listener })
    }

    /// Accepts a new incoming connection from this listener.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn accept<'emma>(
        &'emma self,
        emma: &'emma Emma,
    ) -> Result<
        Pin<
            Box<
                dyn EmmaFuture<Output = Result<(TcpStream, Option<net::SocketAddr>)>>
                    + 'emma
                    + Unpin,
            >,
        >,
    > {
        let fut = op::Op::async_accept(self.inner.as_raw_fd(), emma)?;
        let fut = Map::new(fut, |ret: Result<Ready>| {
            ret.map(|ready| {
                let fd = ready.uring_res as _;
                let stream = unsafe { TcpStream::from_raw_fd(fd) };

                (stream, None) // currently ignore socket addr
            })
        });

        Ok(Box::pin(fut))
    }
}
