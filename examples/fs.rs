use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::sync::Arc;

fn main() -> io::Result<()> {
    let f = File::open("README.md")?;
    let f = emma::fs::File::from_std(f);

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let emma = emma::Builder::new().build()?;
    let emma = Arc::new(RefCell::new(emma));

    rt.block_on(task(emma, f))
}

async fn task(emma: Arc<RefCell<emma::Emma>>, f: emma::fs::File) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    let read_fut = f.async_read(emma.clone(), &mut buf);
    let wake_fut = emma::EmmaReactor::from_emma(&emma);
    futures::try_join!(read_fut, wake_fut).map(|_| {
        println!("{}", String::from_utf8_lossy(&buf));
    })
}
