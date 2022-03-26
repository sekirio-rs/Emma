use crate::io::EmmaBuf;
use crate::io::{op, read};
use crate::Emma;
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
        &'emma mut self,
        emma: &'emma Emma,
        buf: &'emma mut T,
    ) -> Result<Pin<Box<op::Op<'emma, read::Read<'emma, T>>>>> {
        let fut = op::Op::async_read(self.std.as_raw_fd(), emma, buf)?;
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
            let fut = op::Op::async_read(file.std.as_raw_fd(), emma, buf)?;
            let boxed_fut = Box::pin(fut);

            futs.push(boxed_fut);
        }

        Ok(futs)
    }
}
