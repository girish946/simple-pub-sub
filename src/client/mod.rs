use crate::message;
use crate::stream;
use log::{error, info, trace};
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

/// default implementation for callback function
pub fn on_message(topic: String, message: Vec<u8>) {
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
                Ok(buf)
            }
            None => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet.",
            )),
        }
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

    pub async fn query(self, topic: String) -> Result<String, tokio::io::Error> {
        let msg: message::Msg = message::Msg::new(
            message::PktType::QUERY,
            topic,
            Some("a".to_string().as_bytes().to_vec()),
        );
        trace!("msg: {:?}", msg);

        match self.stream {
            Some(mut s) => match s.write_all(&msg.bytes()).await {
                Ok(_) => match stream::read_message(&mut s).await {
                    Ok(msg) => match String::from_utf8(msg.message.clone()) {
                        Ok(msg_str) => Ok(msg_str),
                        Err(e) => Err(tokio::io::Error::new(
                            tokio::io::ErrorKind::Other,
                            e.to_string(),
                        )),
                    },
                    Err(e) => Err(e),
                },
                Err(e) => Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::Other,
                    e.to_string(),
                )),
            },
            None => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet.".to_string(),
            )),
        }
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
                    match stream::read_message(&mut s).await {
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
            None => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet.".to_string(),
            )),
        }
    }
}
