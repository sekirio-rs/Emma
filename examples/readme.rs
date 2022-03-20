use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    let f = File::open("README.md")?;
    let f = emma::fs::File::from_std(f);

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let emma = emma::Builder::new().build().unwrap();

    rt.block_on(task(emma, f))
}

async fn task(emma: emma::Emma, f: emma::fs::File) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    let read_fut = f.async_read(&emma, &mut buf).unwrap();
    let wake_fut = emma::EmmaReactor::from_emma(&emma);
    futures::try_join!(read_fut, wake_fut)
        .map(|_| {
            println!("{}", String::from_utf8_lossy(&buf));
        })
        .map_err(|e| e.as_io_error())
}
