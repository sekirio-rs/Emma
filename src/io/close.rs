use super::op::Op;
use crate::{Emma, Result};
use io_uring::{opcode, types};
use std::os::unix::io::RawFd;

pub struct Close {
    _fd: RawFd,
}

impl<'emma> Op<'emma, Close> {
    pub(crate) fn async_close(emma: &'emma Emma, fd: RawFd) -> Result<Op<'emma, Close>> {
        Op::async_op(emma, move |token| {
            let entry = opcode::Close::new(types::Fd(fd))
                .build()
                .user_data(token as _);

            (entry, Close { _fd: fd })
        })
    }
}
