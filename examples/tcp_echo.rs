use emma::io::EmmaFuture;
use emma::net::tcp::listener::TcpListener;
use emma::net::tcp::stream::TcpStream;
use emma::Emma;
use emma::Join;
use emma::Reactor;
use std::io;

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let emma = emma::Builder::new().build().unwrap();

        let listener = TcpListener::bind("127.0.0.1:3344").unwrap();

        let stream = accept(&emma, &listener).await.unwrap();

        let mut buf = [0u8; 1024];

        loop {
            // recv
            let n = recv(&emma, &mut buf, &stream).await.unwrap();

            if n == 0 {
                return Ok(());
            }

            send(&emma, &buf, &stream).await.unwrap();
        }
    })
}

async fn accept(emma: &Emma, listener: &TcpListener) -> io::Result<TcpStream> {
    let accept_fut = listener.accept(emma).map_err(|e| e.as_io_error())?;
    let token = accept_fut.as_ref().__token();

    let reactor = Reactor::new(emma);
    let mut join_fut = Join::new(reactor);

    join_fut.as_mut().join(accept_fut);

    let (stream, _) = join_fut
        .await
        .map(|mut ret| ret.remove(&token).unwrap().unwrap())
        .map_err(|e| e.as_io_error())?;

    Ok(stream)
}

async fn recv(emma: &Emma, buf: &mut [u8; 1024], stream: &TcpStream) -> io::Result<usize> {
    let recv_fut = stream.recv(emma, buf).map_err(|e| e.as_io_error())?;
    let token = recv_fut.as_ref().__token();

    let mut join_fut = Join::new(Reactor::new(emma));

    join_fut.as_mut().join(recv_fut);

    let res = join_fut
        .await
        .map(|mut ret| ret.remove(&token).unwrap().unwrap().uring_res as _)
        .map_err(|e| e.as_io_error())?;

    Ok(res)
}

async fn send(emma: &Emma, buf: &[u8; 1024], stream: &TcpStream) -> io::Result<()> {
    let send_fut = stream.send(emma, buf).map_err(|e| e.as_io_error())?;

    let mut join_fut = Join::new(Reactor::new(emma));

    join_fut.as_mut().join(send_fut);

    let _ = join_fut.await.map_err(|e| e.as_io_error())?;

    Ok(())
}
