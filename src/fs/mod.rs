//! Implementation of asynchronous file system operation
//! Based on io_uring
use super::Emma;
use crate::io::EmmaBuf;
use crate::io::{op, read};
use crate::Result;
use std::fs::File as StdFile;
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;

pub struct File {
    // todo
    std: Arc<StdFile>,
}

impl File {
    pub fn from_std(f: StdFile) -> Self {
        Self { std: Arc::new(f) }
    }

    pub fn as_std(&self) -> &StdFile {
        self.std.as_ref()
    }

    pub fn read<'emma, T: EmmaBuf>(
        &mut self,
        emma: &'emma Emma,
        buf: &'emma mut T,
    ) -> Result<Pin<Box<op::Op<'emma, read::Read<'emma, T>>>>> {
        let fut = op::Op::async_read(self.std.as_raw_fd(), emma, buf)?;
        let boxed_fut = Box::pin(fut);

        Ok(boxed_fut)
    }
}
