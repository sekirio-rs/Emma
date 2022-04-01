use super::op::Op;
use super::EmmaBuf;
use crate::Emma;
use crate::Result;
use io_uring::{opcode, types};
use std::os::unix::io::RawFd;

pub struct Recv<'recv, T> {
    fd: RawFd,
    buf: &'recv mut T,
}

impl<'recv, 'emma, T: EmmaBuf> Op<'emma, Recv<'recv, T>> {
    pub fn async_recv(
        fd: RawFd,
        emma: &'emma Emma,
        buf: &'recv mut T,
    ) -> Result<Op<'emma, Recv<'recv, T>>> {
        Op::async_op(emma, move |token| {
            let entry = opcode::Recv::new(types::Fd(fd), buf.mut_ptr(), buf.bytes() as u32)
                .build()
                .user_data(token as _);

            (entry, Recv { fd, buf })
        })
    }
}
