use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    let f = File::open("readme.rs")?;
    let f = emma::fs::File::open(f);

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let mut emma = emma::Builder::new().build()?;

    rt.block_on(async move {
        let mut buf = Vec::<u8>::with_capacity(1024);
        let _ = f.async_read(&mut emma, &mut buf).await?;
        Ok(())
    })
}
