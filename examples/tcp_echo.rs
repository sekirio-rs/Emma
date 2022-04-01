use emma::net::tcp::listener::TcpListener;
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

        let accept_fut = listener.accept(&emma).unwrap();
        let token = accept_fut.as_ref().__token();

        let reactor = Reactor::new(&emma);
        let mut join_fut = Join::new(reactor);

        join_fut.as_mut().join(accept_fut);

        let (_stream, _) = join_fut
            .await
            .map(|mut ret| ret.remove(&token).unwrap().unwrap())
            .unwrap();

        println!("accepted");
        Ok(())
    })
}
