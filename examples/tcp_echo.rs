use emma::{alias::*, net::tcp::listener::TcpListener};
use std::io;

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let emma = emma::Builder::new().entries(1024).build()?;

        let listener = TcpListener::bind("127.0.0.1:3344")?;

        let stream = accept_socket(&emma, &listener).await?;

        let mut buf = [0u8; 1024];

        loop {
            // recv
            let n = recv_msg(&emma, &mut buf, &stream).await?;

            if n == 0 {
                return Ok(());
            }

            send_msg(&emma, &buf, &stream).await?;
        }
    })
}
