use super::super::unix;
use super::stream::TcpStream;
use crate::futures::map::Map;
use crate::io::op::{self, Ready};
use crate::io::EmmaFuture;
use crate::Emma;
use crate::EmmaError;
use crate::Result;
use std::io;
use std::net;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::pin::Pin;

const BACKLOG: u32 = 1024;

pub struct TcpListener {
    inner: net::TcpListener,
}

impl TcpListener {
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
