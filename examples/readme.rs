use emma::alias::*;
use std::io;

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let emma = emma::Builder::new().build()?;

        let mut f = open_file(&emma, "README.md").await?;

        let mut buf = [0u8; 1024];

        read_file(&emma, &mut f, &mut buf).await?;

        println!("{}", String::from_utf8_lossy(&buf));

        Ok(())
    })
}
