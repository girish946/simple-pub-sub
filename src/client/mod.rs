use crate::message;
use log::{debug, error, info, trace, warn};
use std::io::ErrorKind;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
type Callback = fn(String, Vec<u8>);
#[derive(Debug)]
pub struct Client {
    pub server: String,
    pub port: u16,
    stream: Option<TcpStream>,
    callback: Option<Callback>,
}

fn on_message(topic: String, message: Vec<u8>) {
    match String::from_utf8(message.clone()) {
        Ok(msg_str) => {
            info!("topic: {} message: {}", topic, msg_str);
        }
        Err(_) => {
            info!("topic: {} message: {:?}", topic, message);
        }
    };
}

impl Client {
    pub fn new(server: String, port: u16) -> Client {
        Client {
            server,
            port,
            stream: None,
            callback: None,
        }
    }

    pub fn on_message(&mut self, callback: Callback) {
        self.callback = Some(callback)
    }

    pub async fn connect(&mut self) -> Result<(), tokio::io::Error> {
        let server_url: String = format!("{}:{}", self.server, self.port);
        let stream = TcpStream::connect(server_url).await?;
        self.stream = Some(stream);
        Ok(())
    }

    pub async fn post(self, msg: message::Msg) -> Result<Vec<u8>, tokio::io::Error> {
        match self.stream {
            Some(mut s) => {
                match s.write_all(&msg.bytes()).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("could not send the data to the server: {}", e.to_string());
                        return Err(e);
                    }
                }
                let mut buf: Vec<u8>;
                buf = vec![0; 8];
                trace!("reading the ack:");
                let _ = match s.read(&mut buf).await {
                    Ok(n) => n,
                    Err(e) => {
                        error!("could not retrive ack for published packet");
                        return Err(e);
                    }
                };
                return Ok(buf);
            }
            None => {
                return Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::Other,
                    "client not connected yet.",
                ));
            }
        };
    }

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
        let message_position: usize =
            ((8 + header.topic_length) as u16 + header.message_length).into();

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
    pub async fn publish(self, topic: String, message: Vec<u8>) -> Result<(), tokio::io::Error> {
        let msg: message::Msg = message::Msg::new(message::PktType::PUBLISH, topic, Some(message));
        trace!("msg: {:?}", msg);

        let buf = match self.post(msg).await {
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
    pub async fn subscribe(self, topic: String) -> Result<(), tokio::io::Error> {
        match self.stream {
            Some(mut s) => {
                let msg: message::Msg = message::Msg::new(message::PktType::SUBSCRIBE, topic, None);
                trace!("msg: {:?}", msg);

                match s.write_all(&msg.bytes()).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("could not send the data to the server: {}", e.to_string());
                        return Err(tokio::io::Error::new(
                            tokio::io::ErrorKind::Other,
                            e.to_string(),
                        ));
                    }
                };
                loop {
                    match Self::read_message(&mut s).await {
                        Ok(m) => match self.callback {
                            Some(fn_) => {
                                fn_(m.topic, m.message);
                            }
                            None => on_message(m.topic, m.message),
                        },
                        Err(e) => {
                            error!("could not read message: {}", e.to_string());
                            break;
                        }
                    };
                }

                Ok(())
            }
            None => {
                return Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::Other,
                    "client not connected yet.".to_string(),
                ));
            }
        }
    }
}
