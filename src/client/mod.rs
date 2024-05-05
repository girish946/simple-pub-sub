use crate::message;
use crate::stream;
use log::{error, info, trace};
use std::io::ErrorKind;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    net::UnixStream,
};

/// Simple pub sub Client for Tcp connection
#[derive(Debug, Clone)]
pub struct PubSubTcpClient {
    pub server: String,
    pub port: u16,
}

/// Simple pub sub Client for Unix connection
#[derive(Debug, Clone)]
pub struct PubSubUnixClient {
    pub path: String,
}

/// Simple pub sub Client
#[derive(Debug, Clone)]
pub enum PubSubClient {
    Tcp(PubSubTcpClient),
    Unix(PubSubUnixClient),
}

/// Stream for Tcp and Unix connection
#[derive(Debug)]
pub enum StreamType {
    Tcp(TcpStream),
    Unix(UnixStream),
}

/// on_message callback function
type Callback = fn(String, Vec<u8>);

/// Simple pub sub Client
#[derive(Debug)]
pub struct Client {
    pub client_type: PubSubClient,
    stream: Option<StreamType>,
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
    /// Creates a new instance of `Client`
    pub fn new(client_type: PubSubClient) -> Client {
        Client {
            client_type,
            stream: None,
            callback: None,
        }
    }

    /// Sets the on_message callback function
    pub fn on_message(&mut self, callback: Callback) {
        self.callback = Some(callback)
    }

    /// Connects to the server
    pub async fn connect(&mut self) -> Result<(), tokio::io::Error> {
        match self.client_type.clone() {
            PubSubClient::Tcp(tcp_client) => {
                let server_url: String = format!("{}:{}", tcp_client.server, tcp_client.port);
                let stream = TcpStream::connect(server_url).await?;
                self.stream = Some(StreamType::Tcp(stream));
            }
            PubSubClient::Unix(unix_stream) => {
                let path = unix_stream.path;
                let stream = UnixStream::connect(path).await?;
                self.stream = Some(StreamType::Unix(stream));
            }
        }
        Ok(())
    }

    /// Sends the message to the given server and returns the ack
    /// the server could be either a tcp or unix server
    pub async fn post(self, msg: message::Msg) -> Result<Vec<u8>, tokio::io::Error> {
        match self.stream {
            Some(s) => {
                match s {
                    StreamType::Tcp(mut tcp_stream) => {
                        match tcp_stream.write_all(&msg.bytes()).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("could not send the data to the server: {}", e.to_string());
                                return Err(e);
                            }
                        }
                        let mut buf: Vec<u8>;
                        buf = vec![0; 8];
                        trace!("reading the ack:");
                        let _ = match tcp_stream.read(&mut buf).await {
                            Ok(n) => n,
                            Err(e) => {
                                error!("could not retrive ack for published packet");
                                return Err(e);
                            }
                        };
                        return Ok(buf);
                    }
                    StreamType::Unix(mut unix_stream) => {
                        match unix_stream.write_all(&msg.bytes()).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("could not send the data to the server: {}", e.to_string());
                                return Err(e);
                            }
                        }
                        let mut buf: Vec<u8>;
                        buf = vec![0; 8];
                        trace!("reading the ack:");
                        let _ = match unix_stream.read(&mut buf).await {
                            Ok(n) => n,
                            Err(e) => {
                                error!("could not retrive ack for published packet");
                                return Err(e);
                            }
                        };
                        return Ok(buf);
                    }
                };
            }
            None => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet.",
            )),
        }
    }

    /// Publishes the message to the given topic
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

    /// Sends the query message to the server
    pub async fn query(self, topic: String) -> Result<String, tokio::io::Error> {
        let msg: message::Msg = message::Msg::new(
            message::PktType::QUERY,
            topic,
            Some("a".to_string().as_bytes().to_vec()),
        );
        trace!("msg: {:?}", msg);

        match self.stream {
            Some(s) => match s {
                StreamType::Tcp(mut tcp_stream) => match tcp_stream.write_all(&msg.bytes()).await {
                    Ok(_) => match stream::read_message(&mut tcp_stream).await {
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
                StreamType::Unix(mut unix_stream) => {
                    match unix_stream.write_all(&msg.bytes()).await {
                        Ok(_) => match stream::read_message(&mut unix_stream).await {
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
                    }
                }
            },
            None => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet.".to_string(),
            )),
        }
    }

    /// subscribes to the given topic
    pub async fn subscribe(self, topic: String) -> Result<(), tokio::io::Error> {
        match self.stream {
            Some(s) => {
                let msg: message::Msg = message::Msg::new(message::PktType::SUBSCRIBE, topic, None);
                trace!("msg: {:?}", msg);

                match s {
                    StreamType::Tcp(mut tcp_stream) => {
                        match tcp_stream.write_all(&msg.bytes()).await {
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
                            match stream::read_message(&mut tcp_stream).await {
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
                    }
                    StreamType::Unix(mut unix_stream) => {
                        match unix_stream.write_all(&msg.bytes()).await {
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
                            match stream::read_message(&mut unix_stream).await {
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
                    }
                };

                Ok(())
            }
            None => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet.".to_string(),
            )),
        }
    }
}
