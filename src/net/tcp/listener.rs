use super::super::unix;
use crate::EmmaError;
use crate::Result;
use std::io;
use std::net;
use std::os::unix::io::{AsRawFd, FromRawFd};

const BACKLOG: u32 = 1024;

pub struct TcpListener {
    inner: net::TcpListener,
}

impl TcpListener {
    pub fn bind<A: net::ToSocketAddrs>(addr: A) -> Result<Self> {
        let addr = addr
            .to_socket_addrs()
            .map_err(|e| EmmaError::IoError(e))?
            .next()
            .ok_or_else(|| {
                EmmaError::IoError(io::Error::new(io::ErrorKind::Other, "invalid ip address"))
            })?;

        let socket = unix::new_socket(addr)?;

        unix::set_reuseaddr(socket, true)?;

        unix::bind(socket, addr)?;

        unix::listen(socket, BACKLOG)?;

        let listener = unsafe { net::TcpListener::from_raw_fd(socket) };

        Ok(Self { inner: listener })
    }
}
