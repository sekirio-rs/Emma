use emma::fs::File as EmmaFile;
use std::fs::File as StdFile;
use std::io;
use std::time;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;

const PATH: &str = "README.md";
const BUFFER_LEN: usize = 1024;

fn main() -> io::Result<()> {
    let cost_emma = bench_emma()?;
    let cost_tokio = bench_tokio()?;

    println!("cost emma: {} vs tokio: {}", cost_emma, cost_tokio);

    Ok(())
}

fn bench_emma() -> io::Result<u128> {
    let start = time::Instant::now();

    let rt = tokio::runtime::Builder::new_current_thread().build()?;

    let _ = rt.block_on(async move {
        let emma = emma::Builder::new().build().unwrap();

        let mut f0 = EmmaFile::from_std(StdFile::open(PATH)?);
        let mut f1 = EmmaFile::from_std(StdFile::open(PATH)?);
        let mut f2 = EmmaFile::from_std(StdFile::open(PATH)?);
        let mut f3 = EmmaFile::from_std(StdFile::open(PATH)?);
        let mut f4 = EmmaFile::from_std(StdFile::open(PATH)?);
        let mut f5 = EmmaFile::from_std(StdFile::open(PATH)?);
        let mut f6 = EmmaFile::from_std(StdFile::open(PATH)?);
        let mut f7 = EmmaFile::from_std(StdFile::open(PATH)?);
        let mut f8 = EmmaFile::from_std(StdFile::open(PATH)?);
        let mut f9 = EmmaFile::from_std(StdFile::open(PATH)?);

        let mut buf0 = [0u8; BUFFER_LEN];
        let mut buf1 = [0u8; BUFFER_LEN];
        let mut buf2 = [0u8; BUFFER_LEN];
        let mut buf3 = [0u8; BUFFER_LEN];
        let mut buf4 = [0u8; BUFFER_LEN];
        let mut buf5 = [0u8; BUFFER_LEN];
        let mut buf6 = [0u8; BUFFER_LEN];
        let mut buf7 = [0u8; BUFFER_LEN];
        let mut buf8 = [0u8; BUFFER_LEN];
        let mut buf9 = [0u8; BUFFER_LEN];

        let read_fut0 = f0.read(&emma, &mut buf0).unwrap();
        let read_fut1 = f1.read(&emma, &mut buf1).unwrap();
        let read_fut2 = f2.read(&emma, &mut buf2).unwrap();
        let read_fut3 = f3.read(&emma, &mut buf3).unwrap();
        let read_fut4 = f4.read(&emma, &mut buf4).unwrap();
        let read_fut5 = f5.read(&emma, &mut buf5).unwrap();
        let read_fut6 = f6.read(&emma, &mut buf6).unwrap();
        let read_fut7 = f7.read(&emma, &mut buf7).unwrap();
        let read_fut8 = f8.read(&emma, &mut buf8).unwrap();
        let read_fut9 = f9.read(&emma, &mut buf9).unwrap();

        let wake_fut = emma::EmmaReactor::from_emma(&emma);

        futures::try_join!(
            read_fut0, read_fut1, read_fut2, read_fut3, read_fut4, read_fut5, read_fut6, read_fut7,
            read_fut8, read_fut9, wake_fut
        )
        .map(|_| {})
        .map_err(|e| e.as_io_error())
    })?;

    let cost = start.elapsed().as_micros();

    Ok(cost)
}

fn bench_tokio() -> io::Result<u128> {
    let start = time::Instant::now();

    let rt = tokio::runtime::Builder::new_current_thread().build()?;

    let _ = rt.block_on(async move {
        let mut f0 = TokioFile::open(PATH).await?;
        let mut f1 = TokioFile::open(PATH).await?;
        let mut f2 = TokioFile::open(PATH).await?;
        let mut f3 = TokioFile::open(PATH).await?;
        let mut f4 = TokioFile::open(PATH).await?;
        let mut f5 = TokioFile::open(PATH).await?;
        let mut f6 = TokioFile::open(PATH).await?;
        let mut f7 = TokioFile::open(PATH).await?;
        let mut f8 = TokioFile::open(PATH).await?;
        let mut f9 = TokioFile::open(PATH).await?;

        let mut buf0 = [0u8; BUFFER_LEN];
        let mut buf1 = [0u8; BUFFER_LEN];
        let mut buf2 = [0u8; BUFFER_LEN];
        let mut buf3 = [0u8; BUFFER_LEN];
        let mut buf4 = [0u8; BUFFER_LEN];
        let mut buf5 = [0u8; BUFFER_LEN];
        let mut buf6 = [0u8; BUFFER_LEN];
        let mut buf7 = [0u8; BUFFER_LEN];
        let mut buf8 = [0u8; BUFFER_LEN];
        let mut buf9 = [0u8; BUFFER_LEN];

        let read_fut0 = f0.read(&mut buf0);
        let read_fut1 = f1.read(&mut buf1);
        let read_fut2 = f2.read(&mut buf2);
        let read_fut3 = f3.read(&mut buf3);
        let read_fut4 = f4.read(&mut buf4);
        let read_fut5 = f5.read(&mut buf5);
        let read_fut6 = f6.read(&mut buf6);
        let read_fut7 = f7.read(&mut buf7);
        let read_fut8 = f8.read(&mut buf8);
        let read_fut9 = f9.read(&mut buf9);

        futures::try_join!(
            read_fut0, read_fut1, read_fut2, read_fut3, read_fut4, read_fut5, read_fut6, read_fut7,
            read_fut8, read_fut9
        )
        .map(|_| {})
    })?;

    let cost = start.elapsed().as_micros();

    Ok(cost)
}
