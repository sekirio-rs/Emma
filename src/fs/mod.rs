//! Implementation of asynchronous file system operation
//! Based on io_uring
use super::Emma;
use crate::io::op;
use crate::io::EmmaBuf;
use crate::Result;
use futures::{future::BoxFuture, TryFutureExt};
use std::fs::File as StdFile;
use std::os::unix::io::AsRawFd;
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
    ) -> Result<BoxFuture<'emma, Result<usize>>> {
        let fut = op::Op::async_read(self.std.as_raw_fd(), emma, buf)?;
        let fut = fut.map_ok(|ready| ready.uring_res as usize);
        let boxed_fut = Box::pin(fut);

        Ok(boxed_fut)
    }
}
