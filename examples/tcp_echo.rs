use emma::{alias::*, net::tcp::listener::TcpListener};
use std::io;

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let emma = emma::Builder::new().build().unwrap();

        let listener = TcpListener::bind("127.0.0.1:3344").unwrap();

        let stream = accept_socket(&emma, &listener).await.unwrap();

        let mut buf = [0u8; 1024];

        loop {
            // recv
            let n = recv_msg(&emma, &mut buf, &stream).await.unwrap();

            if n == 0 {
                return Ok(());
            }

            send_msg(&emma, &buf, &stream).await.unwrap();
        }
    })
}
