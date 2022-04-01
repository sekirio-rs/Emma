// todo: ref
use crate::EmmaError;
use crate::Result;
use std::io;
use std::mem::size_of;
use std::net;

macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let ret = unsafe { libc::$fn($($arg, )*) };

        if ret == -1 {
            Err(EmmaError::IoError(io::Error::last_os_error()))
        } else {
            Ok(ret)
        }
    }};
}

pub(crate) fn new_socket(addr: net::SocketAddr) -> Result<libc::c_int> {
    let domain = match addr {
        net::SocketAddr::V4(_) => libc::AF_INET,
        net::SocketAddr::V6(_) => libc::AF_INET6,
    };

    let type_ = libc::SOCK_STREAM | libc::SOCK_CLOEXEC;

    let socket = syscall!(socket(domain, type_, 0))?;

    Ok(socket)
}

pub(crate) fn set_reuseaddr(socket: libc::c_int, reuseadr: bool) -> Result<()> {
    let val: libc::c_int = if reuseadr { 1 } else { 0 };
    syscall!(setsockopt(
        socket,
        libc::SOL_SOCKET,
        libc::SO_REUSEADDR,
        &val as *const libc::c_int as *const libc::c_void,
        size_of::<libc::c_int>() as libc::socklen_t,
    ))?;

    Ok(())
}

#[repr(C)]
pub(crate) union SocketAddrCRepr {
    v4: libc::sockaddr_in,
    v6: libc::sockaddr_in6,
}

impl SocketAddrCRepr {
    pub(crate) fn as_ptr(&self) -> *const libc::sockaddr {
        self as *const _ as *const libc::sockaddr
    }
}

pub(crate) fn socket_addr(addr: &net::SocketAddr) -> (SocketAddrCRepr, libc::socklen_t) {
    match addr {
        net::SocketAddr::V4(ref addr) => {
            let sin_addr = libc::in_addr {
                s_addr: u32::from_ne_bytes(addr.ip().octets()),
            };

            let sockaddr_in = libc::sockaddr_in {
                sin_family: libc::AF_INET as libc::sa_family_t,
                sin_port: addr.port().to_be(),
                sin_addr,
                sin_zero: [0; 8],
            };

            let sockaddr = SocketAddrCRepr { v4: sockaddr_in };
            let socklen = size_of::<libc::sockaddr_in>() as libc::socklen_t;

            (sockaddr, socklen)
        }
        net::SocketAddr::V6(ref addr) => {
            let sockaddr_in6 = libc::sockaddr_in6 {
                sin6_family: libc::AF_INET6 as libc::sa_family_t,
                sin6_port: addr.port().to_be(),
                sin6_addr: libc::in6_addr {
                    s6_addr: addr.ip().octets(),
                },
                sin6_flowinfo: addr.flowinfo(),
                sin6_scope_id: addr.scope_id(),
            };

            let sockaddr = SocketAddrCRepr { v6: sockaddr_in6 };
            let socklen = size_of::<libc::sockaddr_in6>() as libc::socklen_t;

            (sockaddr, socklen)
        }
    }
}

pub(crate) fn bind(socket: libc::c_int, addr: net::SocketAddr) -> Result<()> {
    let (raw_addr, raw_addr_length) = socket_addr(&addr);
    syscall!(bind(socket, raw_addr.as_ptr(), raw_addr_length))?;

    Ok(())
}

pub(crate) fn listen(socket: libc::c_int, backlog: u32) -> Result<()> {
    let backlog = backlog.try_into().unwrap_or(i32::max_value());
    syscall!(listen(socket, backlog))?;

    Ok(())
}
