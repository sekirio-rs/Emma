use emma::fs::File as EmmaFile;
use emma::futures::then::IThen;
use std::io;

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let emma = emma::Builder::new().build().unwrap();

        let f = create_file(&emma).await?;

        let buf = [0u8; 1024];

        let read_fut = f.write(&emma, &buf).map_err(|e| e.as_io_error())?;

        let write_fut = read_fut.then(|_| f.write(&emma, &buf).unwrap());

        let reactor = emma::Reactor::new(&emma);
        let mut join = emma::Join::new(reactor);

        join.as_mut().join(Box::pin(write_fut));

        Ok(())
    })
}

async fn create_file(emma: &emma::Emma) -> io::Result<EmmaFile> {
    let reactor = emma::Reactor::new(&emma);
    let mut join_fut = emma::Join::new(reactor);

    let open_fut = EmmaFile::create(emma, "TEMP.md").map_err(|e| e.as_io_error())?;

    let _ = join_fut.as_mut().join(open_fut);

    let f = join_fut
        .await
        .map_err(|e| e.as_io_error())?
        .remove(0)
        .unwrap();

    Ok(f)
}
