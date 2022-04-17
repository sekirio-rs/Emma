use crate::futures::map::IMap;
use crate::io::EmmaBuf;
use crate::io::EmmaFuture;
use crate::io::{
    close,
    op::{self, Ready},
    open::OpenFlags,
    read, write,
};
use crate::Emma;
use crate::Result;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::pin::Pin;

pub struct File {
    // currently raw fd
    fd: RawFd,
}

impl File {
    pub fn fram_raw_fd(fd: RawFd) -> Self {
        Self { fd }
    }

    pub fn open<'emma, P: AsRef<Path>>(
        emma: &'emma Emma,
        path: P,
    ) -> Result<Pin<Box<dyn EmmaFuture<Output = Result<Self>> + 'emma + Unpin>>> {
        Self::open_inner(emma, path, OpenFlags::READ_ONLY)
    }

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

    pub fn read<'emma, T: EmmaBuf>(
        &'emma mut self,
        emma: &'emma Emma,
        buf: &'emma mut T,
    ) -> Result<Pin<Box<op::Op<'emma, read::Read<'emma, T>>>>> {
        let fut = op::Op::async_read(self.fd, emma, buf)?;
        let boxed_fut = Box::pin(fut);

        Ok(boxed_fut)
    }

    pub fn multi_read<'emma, T: EmmaBuf>(
        files: &mut Vec<Self>,
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

    pub fn write<'emma, T: EmmaBuf + Sync>(
        &'emma mut self,
        emma: &'emma Emma,
        buf: &'emma T,
    ) -> Result<Pin<Box<op::Op<'emma, write::Write<'emma, T>>>>> {
        let fut = op::Op::async_write(self.fd, emma, buf)?;
        let boxed_fut = Box::pin(fut);

        Ok(boxed_fut)
    }

    pub fn close<'emma>(self, emma: &'emma Emma) -> Result<Pin<Box<op::Op<'emma, close::Close>>>> {
        Ok(Box::pin(op::Op::async_close(emma, self.fd)?))
    }
}
