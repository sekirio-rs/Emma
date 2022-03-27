use super::op::Op;
use crate::Emma;
use crate::EmmaError;
use crate::EmmaState;
use crate::Result;
use bitflags::bitflags;
use io_uring::{opcode, types};
use std::ffi::CString;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

pub struct Open {
    dirfd: Option<libc::c_int>,
    path: CString,
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
    pub fn async_open<P: AsRef<Path>>(
        emma: &'emma Emma,
        path: P,
        flags: libc::c_int,
    ) -> Result<Op<'emma, Open>> {
        let token = emma.inner.borrow_mut().slab.insert(EmmaState::Submitted);

        let path = CString::new(path.as_ref().as_os_str().as_bytes())
            .map_err(|e| EmmaError::Other(Box::new(e)))?;

        let entry = opcode::OpenAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr())
            .flags(flags)
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

        let data = Open { dirfd: None, path };

        Ok(Op::new(token, emma, data))
    }
}
