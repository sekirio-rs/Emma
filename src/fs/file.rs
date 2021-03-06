//! Copyright (C) 2022 SKTT1Ryze. All rights reserved.
//!
//! Implementation of Linux io_uring based asynchorous filesystem I/O
//! operations.
use crate::{
    futures::map::IMap,
    io::{
        close,
        op::{self, Ready},
        open::OpenFlags,
        read, write, EmmaBuf, EmmaFuture,
    },
    Emma, Result,
};
use std::{os::unix::io::RawFd, path::Path, pin::Pin};

/// The structure holds a file descriptor.
///
/// It's clever to perform all file related operations via [`File`] and it's
/// methods.
pub struct File {
    // currently raw fd
    fd: RawFd,
}

impl File {
    /// Construct a [`File`] with given file descriptor.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn fram_raw_fd(fd: RawFd) -> Self {
        Self { fd }
    }

    /// Asynchorously open a [`File`] with read-only flags.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn open<'emma, P: AsRef<Path>>(
        emma: &'emma Emma,
        path: P,
    ) -> Result<Pin<Box<dyn EmmaFuture<Output = Result<Self>> + 'emma + Unpin>>> {
        Self::open_inner(emma, path, OpenFlags::READ_ONLY)
    }

    /// Asynchorously ppen a [`File`] with write-only flags and crate a file if
    /// it does not exist.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn create<'emma, P: AsRef<Path>>(
        emma: &'emma Emma,
        path: P,
    ) -> Result<Pin<Box<dyn EmmaFuture<Output = Result<Self>> + 'emma + Unpin>>> {
        Self::open_inner(emma, path, OpenFlags::WRITE_ONLY | OpenFlags::CREAT_TRUNC)
    }

    fn open_inner<'emma, P: AsRef<Path>>(
        emma: &'emma Emma,
        path: P,
        flags: OpenFlags,
    ) -> Result<Pin<Box<dyn EmmaFuture<Output = Result<Self>> + 'emma + Unpin>>> {
        let fut = op::Op::async_open(emma, path, flags)?;
        let fut = fut.map(|ret: Result<Ready>| {
            ret.map(|ready| {
                let fd = ready.uring_res as _;
                Self::fram_raw_fd(fd)
            })
        });

        Ok(Box::pin(fut))
    }

    /// Asynchorously read bytes from currently [`File`] to given buffer.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn read<'emma, T: EmmaBuf>(
        &'emma mut self,
        emma: &'emma Emma,
        buf: &'emma mut T,
    ) -> Result<Pin<Box<op::Op<'emma, read::Read<'emma, T>>>>> {
        let fut = op::Op::async_read(self.fd, emma, buf)?;
        let boxed_fut = Box::pin(fut);

        Ok(boxed_fut)
    }

    /// Asynchorously read bytes from multi [`File`]s to multi given buffers.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn multi_read<'emma, T: EmmaBuf>(
        files: &mut [Self],
        emma: &'emma Emma,
        bufs: &'emma mut Vec<T>,
    ) -> Result<Vec<Pin<Box<op::Op<'emma, read::Read<'emma, T>>>>>> {
        assert_eq!(files.len(), bufs.len());

        let mut futs = Vec::new();
        for (file, buf) in files.iter().zip(bufs) {
            let fut = op::Op::async_read(file.fd, emma, buf)?;
            let boxed_fut = Box::pin(fut);

            futs.push(boxed_fut);
        }

        Ok(futs)
    }

    /// Asynchorously write bytes to currently [`File`] from given buffer.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn write<'emma, T: EmmaBuf + Sync>(
        &'emma mut self,
        emma: &'emma Emma,
        buf: &'emma T,
    ) -> Result<Pin<Box<op::Op<'emma, write::Write<'emma, T>>>>> {
        let fut = op::Op::async_write(self.fd, emma, buf)?;
        let boxed_fut = Box::pin(fut);

        Ok(boxed_fut)
    }

    /// Asynchorously write bytes to multi [`File`] from multi given buffer.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn multi_write<'emma, T: EmmaBuf + Sync>(
        files: &mut [Self],
        emma: &'emma Emma,
        bufs: &'emma Vec<T>,
    ) -> Result<Vec<Pin<Box<op::Op<'emma, write::Write<'emma, T>>>>>> {
        assert_eq!(files.len(), bufs.len());

        let mut futs = Vec::new();
        for (file, buf) in files.iter().zip(bufs) {
            let fut = op::Op::async_write(file.fd, emma, buf)?;
            let boxed_fut = Box::pin(fut);

            futs.push(boxed_fut);
        }

        Ok(futs)
    }

    #[allow(clippy::needless_lifetimes, missing_docs)]
    pub fn close<'emma>(self, emma: &'emma Emma) -> Result<Pin<Box<op::Op<'emma, close::Close>>>> {
        Ok(Box::pin(op::Op::async_close(emma, self.fd)?))
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}
