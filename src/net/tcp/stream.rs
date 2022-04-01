use crate::io::EmmaBuf;
use crate::io::{op, recv, send};
use crate::Emma;
use crate::Result;
use std::net;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::pin::Pin;

pub struct TcpStream {
    // currently [`std::net::TcpStream`]
    inner: net::TcpStream,
}

impl TcpStream {
    pub unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            inner: net::TcpStream::from_raw_fd(fd),
        }
    }

    pub fn recv<'emma, T: EmmaBuf>(
        &'emma self,
        emma: &'emma Emma,
        buf: &'emma mut T,
    ) -> Result<Pin<Box<op::Op<'emma, recv::Recv<'emma, T>>>>> {
        let fut = op::Op::async_recv(self.inner.as_raw_fd(), emma, buf)?;

        Ok(Box::pin(fut))
    }

    pub fn send<'emma, T: EmmaBuf + Sync>(
        &'emma self,
        emma: &'emma Emma,
        buf: &'emma T,
    ) -> Result<Pin<Box<op::Op<'emma, send::Send_<'emma, T>>>>> {
        let fut = op::Op::async_send(self.inner.as_raw_fd(), emma, buf)?;

        Ok(Box::pin(fut))
    }
}
