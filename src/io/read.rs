use super::op::Op;
use super::EmmaBuf;
use crate::Emma;
use crate::Result;
use io_uring::{opcode, types};
use std::os::unix::io::RawFd;

pub struct Read<'read, T> {
    /// currently raw fd
    fd: RawFd,
    /// buf reference
    buf: &'read mut T,
}

impl<'read, 'emma, T: EmmaBuf> Op<'emma, Read<'read, T>> {
    pub fn async_read(
        fd: RawFd,
        emma: &'emma Emma,
        buf: &'read mut T,
    ) -> Result<Op<'emma, Read<'read, T>>> {
        Op::async_op(emma, move |token| {
            let entry = opcode::Read::new(types::Fd(fd), buf.mut_ptr(), buf.bytes() as u32)
                .build()
                .user_data(token as _);

            (entry, Read { fd, buf })
        })
    }
}
