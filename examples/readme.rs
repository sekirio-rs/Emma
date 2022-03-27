use emma::io::EmmaFuture;
use std::io;
use std::os::unix::io::RawFd;

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let emma = emma::Builder::new().build().unwrap();

        let fd = open_file(&emma).await?;

        read_file(&emma, fd).await
    })
}

async fn open_file(emma: &emma::Emma) -> io::Result<RawFd> {
    let reactor = emma::Reactor::new(&emma);
    let mut join_fut = emma::Join::new(reactor);

    let open_fut = emma::fs::File::open(emma, "README.rs").map_err(|e| e.as_io_error())?;
    let token = open_fut.as_ref().__token();

    let _ = join_fut.as_mut().join(open_fut);

    let fd = join_fut
        .await
        .map(|mut ret| ret.remove(&token).unwrap().unwrap().uring_res as _)
        .map_err(|e| e.as_io_error())?;

    Ok(fd)
}

async fn read_file(emma: &emma::Emma, fd: RawFd) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    // let mut buf = vec![0u8; 1024].into_boxed_slice();

    let mut f = emma::fs::File::fram_raw_fd(fd);

    let read_fut = f.read(emma, &mut buf).map_err(|e| e.as_io_error())?;
    let reactor = emma::Reactor::new(emma);
    let mut join_fut = emma::Join::new(reactor);

    let _ = join_fut.as_mut().join(read_fut);

    let _ = join_fut.await.map_err(|e| e.as_io_error())?;

    println!("{}", String::from_utf8_lossy(&buf));
    Ok(())
}
