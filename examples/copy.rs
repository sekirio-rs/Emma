use emma::fs::File as EmmaFile;
use std::io;

const SOURCE: &str = "README.md";
const TARGET: &str = "TEMP.md";

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let emma = emma::Builder::new().build().unwrap();
        let mut buf = [0u8; 1024];

        let (src, dest) = open_file(&emma).await?;

        do_read(&emma, src, &mut buf).await?;

        do_write(&emma, dest, &buf).await
    })
}

async fn open_file(emma: &emma::Emma) -> io::Result<(EmmaFile, EmmaFile)> {
    let reactor = emma::Reactor::new(&emma);
    let mut join_fut = emma::Join::new(reactor);

    let open_fut = EmmaFile::open(emma, SOURCE).map_err(|e| e.as_io_error())?;
    let token1 = open_fut.as_ref().__token();

    let create_fut = EmmaFile::create(emma, TARGET).map_err(|e| e.as_io_error())?;
    let token2 = create_fut.as_ref().__token();

    let _ = join_fut.as_mut().join(open_fut).join(create_fut);

    let f = join_fut
        .await
        .map(|mut ret| {
            (
                ret.remove(&token1).unwrap().unwrap(),
                ret.remove(&token2).unwrap().unwrap(),
            )
        })
        .map_err(|e| e.as_io_error())?;

    Ok(f)
}

async fn do_read(emma: &emma::Emma, mut src: EmmaFile, buf: &mut [u8; 1024]) -> io::Result<()> {
    let read_fut = src.read(emma, buf).map_err(|e| e.as_io_error())?;
    let reactor = emma::Reactor::new(emma);
    let mut join_fut = emma::Join::new(reactor);

    let _ = join_fut.as_mut().join(read_fut);

    let _ = join_fut.await.map_err(|e| e.as_io_error())?;

    Ok(())
}

async fn do_write(emma: &emma::Emma, mut dest: EmmaFile, buf: &[u8; 1024]) -> io::Result<()> {
    let read_fut = dest.write(emma, buf).map_err(|e| e.as_io_error())?;
    let reactor = emma::Reactor::new(emma);
    let mut join_fut = emma::Join::new(reactor);

    let _ = join_fut.as_mut().join(read_fut);

    let _ = join_fut.await.map_err(|e| e.as_io_error())?;

    Ok(())
}
