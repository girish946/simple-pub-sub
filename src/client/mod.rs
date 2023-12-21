use std::io::ErrorKind;

use crate::message;
use log::{debug, error, info, trace, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
pub async fn connect(server_url: String) -> Result<TcpStream, tokio::io::Error> {
    info!("connecting");
    let stream = TcpStream::connect(server_url).await?;
    Ok(stream)
}

pub async fn post(
    stream: &mut tokio::net::TcpStream,
    msg: message::Msg,
) -> Result<Vec<u8>, tokio::io::Error> {
    match stream.write_all(&msg.bytes()).await {
        Ok(_) => {}
        Err(e) => {
            error!("could not send the data to the server: {}", e.to_string());
            return Err(e);
        }
    }
    let mut buf: Vec<u8>;
    buf = vec![0; 8];
    trace!("reading the ack:");
    let _ = match stream.read(&mut buf).await {
        Ok(n) => n,
        Err(e) => {
            error!("could not retrive ack for published packet");
            return Err(e);
        }
    };
    Ok(buf)
}

/// reads a data from a `TcpStream` and returns a `Msg`.
pub async fn read_message(
    socket: &mut TcpStream,
    client_id: String,
) -> Result<message::Msg, String> {
    let mut pkt_buf: Vec<u8>;
    pkt_buf = vec![0; 512];
    info!("reading...");
    info!("the size of buffer is: {}", pkt_buf.len());
    let n = match socket.read(&mut pkt_buf).await {
        Ok(n) => n,
        Err(e) => {
            warn!(
                "error occured while reading from the socket: {}",
                e.to_string()
            );
            return Err("".to_string());
        }
    };

    if n == 0 {
        return Err("".to_string());
    }
    debug!("incoming pkt: {:?}", pkt_buf[..8].to_vec().clone());
    let header: message::Header = match message::Header::from_vec(pkt_buf[..8].to_vec()) {
        Ok(h) => h,
        Err(_e) => {
            error!("could not parse header aborting");

            return Err("".to_string());
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
    let message_position: usize = ((8 + header.topic_length) as u16 + header.message_length).into();

    if 504 - u16::from(header.topic_length) < header.message_length {
        let bytes_remaining = header.message_length - (504 - u16::from(header.topic_length));
        trace!("the message is bigger, reading the remainig chunk");
        trace!("{} bytes remaining", bytes_remaining);

        let mut buf: Vec<u8> = Vec::with_capacity(bytes_remaining.into());
        trace!("reading next bytes");

        let n = match socket.read_buf(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                error!(
                    "client disconnected, could not read data: {}",
                    e.to_string()
                );
                return Err("client_disconnected: ".to_string());
            }
        };

        if n == 0 {
            return Err("".to_string());
        }

        pkt_buf.extend(buf);
    }
    Ok(message::Msg {
        header: header.clone(),
        topic,
        message: pkt_buf[(8 + header.topic_length).into()..message_position].to_vec(),
        channel: None,
        client_id: Some(client_id),
    })
}

pub async fn publish(
    server_url: String,
    topic: String,
    message: Vec<u8>,
) -> Result<(), tokio::io::Error> {
    let mut stream = match connect(server_url).await {
        Ok(s) => s,
        Err(e) => {
            error!("could not connect to the server: {}", e.to_string());
            return Err(e);
        }
    };
    let msg: message::Msg = message::Msg::new(message::PktType::PUBLISH, topic, Some(message));
    trace!("msg: {:?}", msg);

    let buf = match post(&mut stream, msg).await {
        Ok(resp) => resp,
        Err(e) => return Err(e),
    };

    trace!("the raw buffer is: {:?}", buf);
    let resp_: message::Header = match message::Header::from_vec(buf) {
        Ok(resp) => resp,
        Err(e) => {
            return Err(tokio::io::Error::new(ErrorKind::Other, format!("{:?}", e)));
        }
    };
    trace!("{:?}", resp_);

    Ok(())
}

pub async fn subscribe(server_url: String, topic: String) -> Result<(), tokio::io::Error> {
    let mut stream = match connect(server_url).await {
        Ok(s) => s,
        Err(e) => {
            error!("could not connect to the server: {}", e.to_string());
            return Err(e);
        }
    };
    let msg: message::Msg = message::Msg::new(message::PktType::SUBSCRIBE, topic, None);
    trace!("msg: {:?}", msg);

    match stream.write_all(&msg.bytes()).await {
        Ok(_) => {}
        Err(e) => {
            error!("could not send the data to the server: {}", e.to_string());
            return Err(e);
        }
    };

    let _ = tokio::spawn(async move {
        let client_id = uuid::Uuid::new_v4().to_string();
        loop {
            tokio::select! {
                msg = read_message(&mut stream, client_id.clone()) => {
                    match msg {
                        Ok(m)=>{
                            info!("{:?}", m);
                        },
                        Err(e)=>{
                            error!("could not read message: {}", e.to_string());
                        }
                    }
                }
            }
        }
    })
    .await;

    Ok(())
}
