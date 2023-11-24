use crate::message;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn handle_clinet(mut socket: TcpStream) {
    tokio::spawn(async move {
        let mut pkt_buf: Vec<u8>;
        loop {
            pkt_buf = vec![0; 512];
            info!("reading...");
            info!("the size of buffer is: {}", pkt_buf.len());
            let n = match socket.read(&mut pkt_buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!(
                        "error occured while reading from the socket: {}",
                        e.to_string()
                    );
                    return;
                }
            };

            if n == 0 {
                return;
            }
            let header: message::Header = match message::Header::from_vec(pkt_buf[..8].to_vec()) {
                Ok(h) => h,
                Err(e) => {
                    error!("could not parse header aborting");
                    return;
                }
            };
            println!("header is :{:?}", header);
            socket
                .write_all(&pkt_buf)
                .await
                .expect("failed to write data to socket");
        }
    });
}
