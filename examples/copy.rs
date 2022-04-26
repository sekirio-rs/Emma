use emma::alias::*;
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

        let mut src = open_file(&emma, SOURCE).await?;
        let mut dest = open_file(&emma, TARGET).await?;

        read_file(&emma, &mut src, &mut buf).await?;

        write_file(&emma, &mut dest, &buf).await
    })
}
