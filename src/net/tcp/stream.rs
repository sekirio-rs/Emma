use std::net;
use std::os::unix::io::{FromRawFd, RawFd};

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
}
