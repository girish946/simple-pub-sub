use crate::message;
use log::{debug, error, info};
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
            debug!("incoming pkt: {:?}", pkt_buf[..8].to_vec().clone());
            let header: message::Header = match message::Header::from_vec(pkt_buf[..8].to_vec()) {
                Ok(h) => h,
                Err(_e) => {
                    error!("could not parse header aborting");
                    return;
                }
            };
            info!("{:?}", header);
            let message_position: usize =
                ((8 + header.topic_length) as u16 + header.message_length).into();
            socket
                .write_all(&pkt_buf[(8 + header.topic_length).into()..message_position])
                .await
                .expect("failed to write data to socket");
        }
    });
}
