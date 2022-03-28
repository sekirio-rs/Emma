use emma::fs::File as EmmaFile;
use std::io;
use std::time;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;

const PATH: &str = "README.md";
const BUFFER_LEN: usize = 1024;
const BENCH_SIZE: usize = 100;

fn main() -> io::Result<()> {
    let cost_emma = bench_emma()?;
    let cost_tokio = bench_tokio()?;

    println!("cost emma: {} vs tokio: {}", cost_emma, cost_tokio);

    Ok(())
}

fn bench_emma() -> io::Result<u128> {
    async fn open_files(emma: &emma::Emma) -> io::Result<Vec<EmmaFile>> {
        let reactor = emma::Reactor::new(&emma);
        let mut join_fut = emma::Join::new(reactor);

        let mut open_futs = Vec::new();

        for _ in 0..BENCH_SIZE {
            let fut = EmmaFile::open(emma, PATH).map_err(|e| e.as_io_error())?;
            open_futs.push(fut);
        }

        let tokens = open_futs
            .iter()
            .map(|fut| fut.as_ref().__token())
            .collect::<Vec<_>>();

        for fut in open_futs {
            join_fut.as_mut().join(fut);
        }

        join_fut
            .await
            .map(|mut ret| {
                tokens
                    .iter()
                    .map(|token| ret.remove(token).unwrap().unwrap())
                    .collect::<Vec<_>>()
            })
            .map_err(|e| e.as_io_error())
    }

    let start = time::Instant::now();

    let rt = tokio::runtime::Builder::new_current_thread().build()?;

    let _ = rt.block_on(async move {
        let emma = emma::Builder::new().build().unwrap();

        let mut files = open_files(&emma).await?;

        let mut bufs = (0..BENCH_SIZE)
            .into_iter()
            .map(|_| [0u8; BUFFER_LEN])
            .collect::<Vec<[u8; BUFFER_LEN]>>();

        let read_futs = emma::fs::File::multi_read(&mut files, &emma, &mut bufs).unwrap();

        let reactor = emma::Reactor::new(&emma);

        let mut joinned_fut = emma::Join::new(reactor);

        read_futs.into_iter().for_each(|fut| {
            joinned_fut.as_mut().join(fut);
        });

        joinned_fut.await.map(|_| ()).map_err(|e| e.as_io_error())
    })?;

    let cost = start.elapsed().as_micros();

    Ok(cost)
}

fn bench_tokio() -> io::Result<u128> {
    let start = time::Instant::now();

    let rt = tokio::runtime::Builder::new_current_thread().build()?;

    let _ = rt.block_on(async {
        let mut handles = Vec::new();
        for _ in 0..BENCH_SIZE {
            let handle = rt.spawn(async {
                let mut f = TokioFile::open(PATH).await.unwrap();
                let mut buf = [0u8; BUFFER_LEN];

                let _ = f.read(&mut buf).await.unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await.unwrap();
        }
    });

    let cost = start.elapsed().as_micros();

    Ok(cost)
}
