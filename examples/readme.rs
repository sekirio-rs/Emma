use emma::fs::File as EmmaFile;
use emma::io::EmmaFuture;
use std::io;
use std::os::unix::io::RawFd;

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let emma = emma::Builder::new().build().unwrap();

        let f = open_file(&emma).await?;

        read_file(&emma, f).await
    })
}

async fn open_file(emma: &emma::Emma) -> io::Result<EmmaFile> {
    let reactor = emma::Reactor::new(&emma);
    let mut join_fut = emma::Join::new(reactor);

    let open_fut = emma::fs::File::open(emma, "README.md").map_err(|e| e.as_io_error())?;
    let token = open_fut.as_ref().__token();

    let _ = join_fut.as_mut().join(open_fut);

    let f = join_fut
        .await
        .map(|mut ret| ret.remove(&token).unwrap().unwrap())
        .map_err(|e| e.as_io_error())?;

    Ok(f)
}

async fn read_file(emma: &emma::Emma, mut f: EmmaFile) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    // let mut buf = vec![0u8; 1024].into_boxed_slice();

    let read_fut = f.read(emma, &mut buf).map_err(|e| e.as_io_error())?;
    let reactor = emma::Reactor::new(emma);
    let mut join_fut = emma::Join::new(reactor);

    let _ = join_fut.as_mut().join(read_fut);

    let _ = join_fut.await.map_err(|e| e.as_io_error())?;

    println!("{}", String::from_utf8_lossy(&buf));
    Ok(())
}
