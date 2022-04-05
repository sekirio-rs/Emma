use crate::fs::File as EmmaFile;
use crate::futures::*;
use crate::io::EmmaBuf;
use crate::Emma;
use crate::Reactor;
use std::io;

pub async fn open_file(emma: &Emma, path: impl AsRef<std::path::Path>) -> io::Result<EmmaFile> {
    let open_fut = EmmaFile::open(emma, path).map_err(|e| e.as_io_error())?;

    single::Single::new(Reactor::new(emma), open_fut)
        .await
        .map_err(|e| e.as_io_error())?
        .map_err(|e| e.as_io_error())
}

pub async fn read_file<T: EmmaBuf>(
    emma: &Emma,
    file: &mut EmmaFile,
    buf: &mut T,
) -> io::Result<()> {
    let read_fut = file.read(emma, buf).map_err(|e| e.as_io_error())?;

    single::Single::new(Reactor::new(emma), read_fut)
        .await
        .map_err(|e| e.as_io_error())?
        .map_err(|e| e.as_io_error())?;

    Ok(())
}
