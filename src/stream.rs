use crate::message;
use log::{debug, error, trace, warn};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
/// reads a data from a `TcpStream` and returns a `Msg`.
pub async fn read_message(s: &mut TcpStream) -> Result<message::Msg, tokio::io::Error> {
    let mut pkt_buf: Vec<u8>;
    pkt_buf = vec![0; 512];

    trace!("the size of buffer is: {}", pkt_buf.len());
    let n = match s.read(&mut pkt_buf).await {
        Ok(n) => n,
        Err(e) => {
            warn!(
                "error occured while reading from the socket: {}",
                e.to_string()
            );
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };

    if n == 0 {
        return Err(tokio::io::Error::new(
            tokio::io::ErrorKind::Other,
            "error while reading the data from socket".to_string(),
        ));
    }
    debug!("incoming pkt: {:?}", pkt_buf[..8].to_vec().clone());
    let header: message::Header = match message::Header::from_vec(pkt_buf[..8].to_vec()) {
        Ok(h) => h,
        Err(e) => {
            error!("could not parse header aborting");
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                format!("{:?}", e),
            ));
        }
    };
    debug!("{:?}", header);

    let topic: String =
        match String::from_utf8(pkt_buf[8..(8 + header.topic_length).into()].to_vec()) {
            Ok(topic) => topic,
            Err(_e) => {
                error!("unable to parse topic, topic needs to be in utf-8");
                "".to_string()
            }
        };
    let message_position: usize = ((8 + header.topic_length) as u16 + header.message_length).into();

    if 504 - u16::from(header.topic_length) < header.message_length {
        let bytes_remaining = header.message_length - (504 - u16::from(header.topic_length));
        trace!("the message is bigger, reading the remainig chunk");
        trace!("{} bytes remaining", bytes_remaining);

        let mut buf: Vec<u8> = Vec::with_capacity(bytes_remaining.into());
        trace!("reading next bytes");

        let n = match s.read_buf(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                error!(
                    "client disconnected, could not read data: {}",
                    e.to_string()
                );
                return Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::Other,
                    e.to_string(),
                ));
            }
        };

        if n == 0 {
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "error while reading the data from socket".to_string(),
            ));
        }

        pkt_buf.extend(buf);
    }
    Ok(message::Msg {
        header: header.clone(),
        topic,
        message: pkt_buf[(8 + header.topic_length).into()..message_position].to_vec(),
        channel: None,
        client_id: None,
    })
}
