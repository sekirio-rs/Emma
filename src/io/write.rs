//! Copyright (C) 2022 SKTT1Ryze. All rights reserved.
use super::{op::Op, EmmaBuf};
use crate::{Emma, Result};
use io_uring::{opcode, types};
use std::os::unix::io::RawFd;

pub struct Write<'write, T> {
    /// currently raw fd
    _fd: RawFd,
    /// buf reference
    _buf: &'write T,
}

impl<'write, 'emma, T: EmmaBuf + Sync> Op<'emma, Write<'write, T>> {
    pub fn async_write(
        fd: RawFd,
        emma: &'emma Emma,
        buf: &'write T,
    ) -> Result<Op<'emma, Write<'write, T>>> {
        Op::async_op(emma, move |token| {
            let entry = opcode::Write::new(types::Fd(fd), buf.ptr(), buf.bytes() as u32)
                .build()
                .user_data(token as _);

            (entry, Write { _fd: fd, _buf: buf })
        })
    }
}
