//! Copyright (C) 2022 SKTT1Ryze. All rights reserved.
use super::{op::Op, EmmaBuf};
use crate::{Emma, Result};
use io_uring::{opcode, types};
use std::os::unix::io::RawFd;

pub struct Read<'read, T> {
    /// currently raw fd
    _fd: RawFd,
    /// buf reference
    _buf: &'read mut T,
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

            (entry, Read { _fd: fd, _buf: buf })
        })
    }
}
