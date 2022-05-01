use super::op::Op;
use crate::{Emma, EmmaError, Result};
use bitflags::bitflags;
use io_uring::{opcode, types};
use std::{ffi::CString, os::unix::prelude::OsStrExt, path::Path};

pub struct Open {
    _dirfd: Option<libc::c_int>,
    _path: CString,
}

bitflags! {
    pub(crate) struct OpenFlags: libc::c_int {
        // access flags
        const READ_ONLY = libc::O_RDONLY;
        const WRITE_ONLY = libc::O_WRONLY;
        const READ_WRITE = libc::O_RDWR;
        const WRITE_APPEND = libc::O_WRONLY | libc::O_APPEND;
        const RDWR_APEEND = libc::O_RDWR | libc::O_APPEND;
        // creation flags
        const CREATE = libc::O_CREAT;
        const TRUNCATE = libc::O_TRUNC;
        const CREAT_TRUNC = libc::O_CREAT | libc::O_TRUNC;
        const CREATE_NEW = libc::O_CREAT | libc::O_EXCL;
    }
}

impl<'emma> Op<'emma, Open> {
    pub(crate) fn async_open<P: AsRef<Path>>(
        emma: &'emma Emma,
        path: P,
        flags: OpenFlags,
    ) -> Result<Op<'emma, Open>> {
        let path = CString::new(path.as_ref().as_os_str().as_bytes())
            .map_err(|e| EmmaError::Other(Box::new(e)))?;

        Op::async_op(emma, move |token| {
            let entry = opcode::OpenAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr())
                .flags(libc::O_CLOEXEC as i32 | flags.bits())
                .build()
                .user_data(token as _);

            (
                entry,
                Open {
                    _dirfd: None,
                    _path: path,
                },
            )
        })
    }
}
