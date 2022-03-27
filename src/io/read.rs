use super::op::Op;
use super::EmmaBuf;
use crate::error::EmmaError;
use crate::Result;
use crate::{Emma, EmmaState};
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
        // 1. push read sqe to uring submission queue
        // 2. construct [`Read<'_, T>`]
        //
        // todo: offset

        let token = emma.inner.borrow_mut().slab.insert(EmmaState::Submitted);

        let entry = opcode::Read::new(types::Fd(fd), buf.mut_ptr(), buf.bytes() as u32)
            .build()
            .user_data(token as _);

        let mut uring = emma.uring.borrow_mut();

        if uring.submission().is_full() {
            uring.submit().map_err(|e| EmmaError::IoError(e))?; // flush to kernel
        }

        let mut sq = uring.submission();

        unsafe {
            if let Err(e) = sq.push(&entry) {
                return Err(EmmaError::Other(Box::new(e)));
            }
        }

        sq.sync(); // sync to true uring submission queue

        let data = Read { fd, buf };

        Ok(Op::new(token, emma, data))
    }
}
