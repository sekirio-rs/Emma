// currently can not be complied
use std::cell::RefCell;
use std::fs::File;
use std::io;

thread_local! {
    static EMMA: emma::Emma = emma::Builder::new().build().unwrap();
    static BUFFER: RefCell<[u8; 1024]> = RefCell::new([0u8; 1024]);
}

fn main() -> io::Result<()> {
    let f = File::open("README.md")?;
    let f = emma::fs::File::from_std(f);

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let handle = EMMA.with(|e| {
        BUFFER.with(move |b| {
            let buf = &mut *b.borrow_mut();
            task_(e, buf, f)
        })
    });

    let _ = rt.block_on(async { handle.await });

    Ok(())
}

fn task_(
    emma: &'static emma::Emma,
    buf: &'static mut [u8; 1024],
    mut f: emma::fs::File,
) -> tokio::task::JoinHandle<Result<usize, emma::error::EmmaError>> {
    let wake_fut = emma::EmmaReactor::from_emma(emma);
    let read_fut = f.async_read(emma, buf).unwrap();

    let handle = tokio::spawn(read_fut);
    let _ = tokio::spawn(wake_fut);

    handle
}
