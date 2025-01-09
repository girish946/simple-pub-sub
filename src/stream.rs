use crate::message;
use log::{debug, error, trace, warn};
use tokio::io::AsyncReadExt;

const HEADER_SIZE: usize = 8;

/// reads a data from a `TcpStream` and returns a `Msg`.
pub async fn read_message<S>(s: &mut S) -> Result<message::Msg, tokio::io::Error>
where
    S: AsyncReadExt + Unpin + Send,
{
    let mut pkt_buf: Vec<u8>;
    pkt_buf = vec![0; 512];

    trace!("the size of buffer is: {}", pkt_buf.len());
    let n = match s.read(&mut pkt_buf).await {
        Ok(n) => n,
        Err(e) => {
            warn!(
                "error occurred while reading from the socket: {}",
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
    debug!(
        "incoming pkt: {:?}",
        pkt_buf[..HEADER_SIZE].to_vec().clone()
    );
    let header: message::Header = match message::Header::from_vec(pkt_buf[..HEADER_SIZE].to_vec()) {
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

    let topic: String = match String::from_utf8(
        pkt_buf[HEADER_SIZE..(HEADER_SIZE + header.topic_length as usize).into()].to_vec(),
    ) {
        Ok(topic) => topic,
        Err(_e) => {
            error!("unable to parse topic, topic needs to be in utf-8");
            "".to_string()
        }
    };
    let message_position: usize =
        ((HEADER_SIZE + header.topic_length as usize) as u16 + header.message_length).into();

    if 504 - u16::from(header.topic_length) < header.message_length {
        let bytes_remaining = header.message_length - (504 - u16::from(header.topic_length));
        trace!("the message is bigger, reading the remaining chunk");
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
        message: pkt_buf[(HEADER_SIZE + header.topic_length as usize).into()..message_position]
            .to_vec(),
        channel: None,
        client_id: None,
    })
}
