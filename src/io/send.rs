use super::op::Op;
use super::EmmaBuf;
use crate::Emma;
use crate::Result;
use io_uring::{opcode, types};
use std::os::unix::io::RawFd;

pub struct Send_<'send, T> {
    fd: RawFd,
    buf: &'send T,
}

impl<'send, 'emma, T: EmmaBuf + Sync> Op<'emma, Send_<'send, T>> {
    pub fn async_send(
        fd: RawFd,
        emma: &'emma Emma,
        buf: &'send T,
    ) -> Result<Op<'emma, Send_<'send, T>>> {
        Op::async_op(emma, move |token| {
            let entry = opcode::Send::new(types::Fd(fd), buf.ptr(), buf.bytes() as u32)
                .build()
                .user_data(token as _);

            (entry, Send_ { fd, buf })
        })
    }
}
