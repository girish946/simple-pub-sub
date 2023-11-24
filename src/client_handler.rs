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

            let topic: String =
                match String::from_utf8(pkt_buf[8..(8 + header.topic_length).into()].to_vec()) {
                    Ok(topic) => topic,
                    Err(_e) => {
                        error!("unable to parse topic, topic needs to be in utf-8");
                        "".to_string()
                    }
                };
            info!("topic: {}", topic);

            let message_position: usize =
                ((8 + header.topic_length) as u16 + header.message_length).into();

            if 504 - u16::from(header.topic_length) < header.message_length {
                let bytes_remaining =
                    header.message_length - (504 - u16::from(header.topic_length));
                // debug!("the message is bigger, reading the remainig chunk");
                // debug!("{} bytes remaining", bytes_remaining);

                let mut buf: Vec<u8> = Vec::with_capacity(bytes_remaining.into());
                // debug!("reading next bytes");

                let n = socket
                    .read_buf(&mut buf)
                    .await
                    .expect("failed to read data from socket");
                // debug!("reading : {n}");
                if n == 0 {
                    return;
                }

                // debug!("bytes data: {:?}", buf.clone());

                pkt_buf.extend(buf);
            }
            socket
                .write_all(&pkt_buf[(8 + header.topic_length).into()..message_position])
                .await
                .expect("failed to write data to socket");
        }
    });
}
