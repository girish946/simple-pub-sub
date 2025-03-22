use crate::message;
use crate::Header;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use log::{debug, trace};
use tokio::io::AsyncReadExt;

/// reads a data from a `TcpStream` and returns a `Msg`.
pub(crate) async fn read_message<S>(s: &mut S) -> Result<message::Msg>
where
    S: AsyncReadExt + Unpin + Send,
{
    let mut pkt_buf: Vec<u8>;
    pkt_buf = vec![0; 512];

    trace!("the size of buffer is: {}", pkt_buf.len());
    let n = s
        .read(&mut pkt_buf)
        .await
        .context("Error while reading data")?;
    if n == 0 {
        bail!("error while readingdata from the socket");
    }
    debug!("incoming pkt: {:?}", pkt_buf[..8].to_vec().clone());
    let header: Header = Header::try_from(&pkt_buf[..8])?;
    debug!("{:?}", header);

    let topic: String = String::from_utf8(pkt_buf[8..(8 + header.topic_length).into()].to_vec())
        .context("Error while parsing the topic string")?;
    let message_position: usize = ((8 + header.topic_length) as u16 + header.message_length).into();

    if 504 - u16::from(header.topic_length) < header.message_length {
        let bytes_remaining = header.message_length - (504 - u16::from(header.topic_length));
        trace!("the message is bigger, reading the remaining chunk");
        trace!("{} bytes remaining", bytes_remaining);

        let mut buf: Vec<u8> = Vec::with_capacity(bytes_remaining.into());
        trace!("reading next bytes");

        let n = s.read_buf(&mut buf).await?;
        if n == 0 {
            bail!("Error while reading the data from socket");
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
