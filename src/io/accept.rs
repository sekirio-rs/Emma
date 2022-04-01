use super::op::Op;
use crate::Emma;
use crate::Result;
use io_uring::{opcode, types};
use std::mem::size_of;
use std::mem::MaybeUninit;
use std::os::unix::io::RawFd;

// todo: ref
pub struct Accept {
    /// currently raw fd
    fd: RawFd,
    addr: MaybeUninit<libc::sockaddr_storage>,
    addr_len: libc::socklen_t,
}

impl<'emma> Op<'emma, Accept> {
    pub fn async_accept(fd: RawFd, emma: &'emma Emma) -> Result<Op<'emma, Accept>> {
        Op::async_op(emma, move |token| {
            let mut accept = Accept {
                fd,
                addr: MaybeUninit::uninit(),
                addr_len: size_of::<libc::sockaddr_storage>() as libc::socklen_t,
            };

            let entry = opcode::Accept::new(
                types::Fd(fd),
                accept.addr.as_mut_ptr() as *mut _,
                &mut accept.addr_len,
            )
            .build()
            .user_data(token as _);

            (entry, accept)
        })
    }
}
