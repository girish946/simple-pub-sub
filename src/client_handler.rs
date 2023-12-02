use crate::message;
use log::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Sender;

pub async fn read_message(socket: &mut TcpStream) -> Result<message::Msg, String> {
    let mut pkt_buf: Vec<u8>;
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
        // debug!("the message is bigger, reading the remainig chunk");
        // debug!("{} bytes remaining", bytes_remaining);

        let mut buf: Vec<u8> = Vec::with_capacity(bytes_remaining.into());
        // debug!("reading next bytes");

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
    return Ok(message::Msg {
        header: header.clone(),
        topic,
        message: pkt_buf[(8 + header.topic_length).into()..message_position].to_vec(),
        channel: None,
    });
}

pub async fn read_channel_msg(
    chan: Sender<message::Msg>,
) -> Result<message::Msg, tokio::sync::broadcast::error::RecvError> {
    let mut rx = chan.subscribe();
    rx.recv().await
}

pub async fn handle_clinet(mut socket: TcpStream, chan: Sender<message::Msg>) {
    let client_chan: tokio::sync::broadcast::Sender<message::Msg> =
        tokio::sync::broadcast::Sender::new(1);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                msg_ = read_message(&mut socket) =>{
                    match msg_{
                        Ok(mut m)=>{
                            if !m.topic.is_empty() {
                                info!("topic: {}", m.topic);
                                match m.header.pkt_type{
                                    message::PktType::PUBLISH=>{
                                        info!("it's a publish packet");
                                        match chan.send(m.clone()) {
                                            Ok(n) => n,
                                            Err(e) => {
                                                error!("error while checking the topic in the map: {}",e.to_string());
                                                0
                                            }
                                        };
                                    },
                                    message::PktType::SUBSCRIBE=>{
                                        info!("it's a subscribe pkt attaching the channel");
                                        m.channel(client_chan.clone());
                                        match chan.send(m.clone()) {
                                            Ok(n) => n,
                                            Err(e) => {
                                                error!("error while checking the topic in the map: {}",e.to_string());
                                                0
                                            }
                                        };
                                    }
                                    message::PktType::UNSUBSCRIBE=> {
                                        m.channel(client_chan.clone());
                                        match chan.send(m.clone()) {
                                            Ok(n) => n,
                                            Err(e) => {
                                                error!("error while checking the topic in the map: {}", e.to_string());
                                                0
                                            },
                                        };
                                    },
                                    _=>{}
                                };
                           }
                           match message::get_msg_response(m.clone()){
                               Ok(v)=>{
                               socket.write_all(&v).await.expect("could not convert message to bytes");
                               },
                               Err(e)=>{
                                   error!("error while writing the data to the socket: {}", e.to_string());
                               }
                           }

                        },
                        Err(e)=>{
                            error!("error while receiving the message: {}", e);
                            return;
                        }
                    };
                },
                chan_msg = read_channel_msg(client_chan.clone())=>{
                    match chan_msg {
                        Ok(m) => {
                            info!("message received: {:?}, {}", m.topic.clone(), m.message.len());
                            socket
                                    .write_all(&m.bytes())
                                    .await
                                    .expect("failed to write data to socket");
                        },
                        Err(_e) => {},
                    }

                }
            }
        }
    });
}
