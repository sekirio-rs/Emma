use crate::{
    fs::File as EmmaFile,
    futures::*,
    io::EmmaBuf,
    net::tcp::{listener::TcpListener as EmmaListener, stream::TcpStream as EmmaStream},
    Emma, Reactor,
};
use std::io;

pub async fn open_file(emma: &Emma, path: impl AsRef<std::path::Path>) -> io::Result<EmmaFile> {
    let open_fut = EmmaFile::open(emma, path)?;

    Ok(single::Single::new(Reactor::new(emma), open_fut).await??)
}

pub async fn read_file<T: EmmaBuf>(
    emma: &Emma,
    file: &mut EmmaFile,
    buf: &mut T,
) -> io::Result<()> {
    let read_fut = file.read(emma, buf)?;

    single::Single::new(Reactor::new(emma), read_fut).await??;

    Ok(())
}

pub async fn write_file<T: EmmaBuf + Sync>(
    emma: &Emma,
    file: &mut EmmaFile,
    buf: &T,
) -> io::Result<()> {
    let write_fut = file.write(emma, buf)?;

    single::Single::new(Reactor::new(emma), write_fut).await??;

    Ok(())
}

pub async fn accept_socket(emma: &Emma, listener: &EmmaListener) -> io::Result<EmmaStream> {
    let accept_fut = listener.accept(emma)?;

    let (stream, _) = single::Single::new(Reactor::new(emma), accept_fut).await??;

    Ok(stream)
}

pub async fn recv_msg<T: EmmaBuf + ?Sized>(
    emma: &Emma,
    buf: &mut T,
    stream: &EmmaStream,
) -> io::Result<usize> {
    let recv_fut = stream.recv(emma, buf)?;

    let res = single::Single::new(Reactor::new(emma), recv_fut).await??;

    Ok(res.uring_res as usize)
}

pub async fn send_msg<T: EmmaBuf + Sync + ?Sized>(
    emma: &Emma,
    buf: &T,
    stream: &EmmaStream,
) -> io::Result<()> {
    let send_fut = stream.send(emma, buf)?;

    single::Single::new(Reactor::new(emma), send_fut).await??;

    Ok(())
}
