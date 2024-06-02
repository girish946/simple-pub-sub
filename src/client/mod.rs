use crate::message;
use crate::stream;
use log::{error, info, trace};
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    net::UnixStream,
};
use tokio_native_tls::native_tls::{Certificate, TlsConnector};
use tokio_native_tls::TlsStream;

/// Simple pub sub Client for Tcp connection
#[derive(Debug, Clone)]
pub struct PubSubTcpClient {
    pub server: String,
    pub port: u16,
    pub cert: Option<String>,
    pub cert_password: Option<String>,
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
    Tls(TlsStream<TcpStream>),
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

    async fn connect_tls(&mut self, url: String, cert: String) -> Result<(), tokio::io::Error> {
        // Load CA certificate
        let mut file = match File::open(cert) {
            Ok(file) => file,
            Err(e) => {
                error!("unable to open the certificate file: {}", e);
                return Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, e));
            }
        };
        let mut ca_cert = vec![];
        match file.read_to_end(&mut ca_cert) {
            Ok(size) => size,
            Err(e) => {
                error!("unable to read the CA file: {}", e);
                return Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, e));
            }
        };
        let ca_cert = match Certificate::from_pem(&ca_cert) {
            Ok(cert) => cert,
            Err(e) => {
                error!("unable to parse the CA certificate: {}", e);
                return Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, e));
            }
        };

        // Configure TLS
        let connector = match TlsConnector::builder()
            .add_root_certificate(ca_cert)
            .build()
        {
            Ok(connector) => connector,
            Err(e) => {
                error!("cannot create TLS connector: {}", e);
                return Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, e));
            }
        };

        let connector = tokio_native_tls::TlsConnector::from(connector);

        // Connect to the server
        let stream = match TcpStream::connect(url).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("failed to connect to server: {}", e);
                return Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, e));
            }
        };

        self.stream = match connector.connect("localhost", stream).await {
            Ok(stream) => Some(StreamType::Tls(stream)),
            Err(e) => {
                error!("could not establish the tls connection: {}", e);
                return Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, e));
            }
        };
        Ok(())
    }

    /// Connects to the server
    pub async fn connect(&mut self) -> Result<(), tokio::io::Error> {
        match self.client_type.clone() {
            PubSubClient::Tcp(tcp_client) => {
                let server_url: String = format!("{}:{}", tcp_client.server, tcp_client.port);
                if let Some(cert) = tcp_client.cert {
                    self.connect_tls(server_url, cert).await?;
                } else {
                    let stream = TcpStream::connect(server_url).await?;
                    self.stream = Some(StreamType::Tcp(stream));
                }
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
    pub async fn post(&mut self, msg: message::Msg) -> Result<Vec<u8>, tokio::io::Error> {
        self.write(msg.bytes()).await?;
        let mut buf: Vec<u8>;
        buf = vec![0; 8];
        match self.read(&mut buf).await {
            Ok(()) => {
                trace!("buf: {:?}", buf);
                match message::Header::from_vec(buf.clone()) {
                    Ok(resp) => {
                        trace!("resp: {:?}", resp);
                        // read the remaining message
                        let mut buf_buf = Vec::with_capacity(resp.message_length as usize);
                        trace!("reading remaining bytes");
                        match self.read_buf(&mut buf_buf).await {
                            Ok(()) => {
                                buf.extend(buf_buf);
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    Err(e) => {
                        return Err(tokio::io::Error::new(ErrorKind::Other, format!("{:?}", e)));
                    }
                };

                Ok(buf)
            }
            Err(e) => Err(e),
        }
    }

    /// Publishes the message to the given topic
    pub async fn publish(
        &mut self,
        topic: String,
        message: Vec<u8>,
    ) -> Result<(), tokio::io::Error> {
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
    pub async fn query(&mut self, topic: String) -> Result<String, tokio::io::Error> {
        let msg: message::Msg = message::Msg::new(
            message::PktType::QUERY,
            topic,
            Some(" ".to_string().as_bytes().to_vec()),
        );
        trace!("msg: {:?}", msg);

        self.write(msg.bytes()).await?;
        let msg = self.read_message().await;
        match msg {
            Ok(m) => match String::from_utf8(m.message) {
                Ok(s) => Ok(s),
                Err(e) => Err(tokio::io::Error::new(ErrorKind::Other, e.to_string())),
            },
            Err(e) => Err(e),
        }
    }

    /// subscribes to the given topic
    pub async fn subscribe(&mut self, topic: String) -> Result<(), tokio::io::Error> {
        let msg: message::Msg = message::Msg::new(message::PktType::SUBSCRIBE, topic, None);
        trace!("msg: {:?}", msg);
        self.write(msg.bytes()).await?;
        if let Some(callback) = self.callback {
            loop {
                match self.read_message().await {
                    Ok(m) => {
                        callback(m.topic, m.message);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        } else {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "no callback function is set".to_string(),
            ))
        }
    }
    pub async fn write(&mut self, message: Vec<u8>) -> Result<(), tokio::io::Error> {
        if let Some(stream) = &mut self.stream {
            match stream {
                StreamType::Tls(tls_stream) => match tls_stream.write_all(&message).await {
                    Ok(size) => {
                        trace!("{:?} bytes written", size);
                        Ok(())
                    }
                    Err(e) => Err(tokio::io::Error::new(
                        tokio::io::ErrorKind::Other,
                        e.to_string(),
                    )),
                },
                StreamType::Tcp(ref mut tcp_stream) => match tcp_stream.write_all(&message).await {
                    Ok(size) => {
                        trace!("{:?} bytes written", size);
                        Ok(())
                    }
                    Err(e) => Err(e),
                },

                StreamType::Unix(ref mut unix_stream) => {
                    match unix_stream.write_all(&message).await {
                        Ok(size) => {
                            trace!("{:?} bytes written", size);
                            Ok(())
                        }

                        Err(e) => Err(e),
                    }
                }
            }
        } else {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet".to_string(),
            ))
        }
    }

    pub async fn read(&mut self, message: &mut [u8]) -> Result<(), tokio::io::Error> {
        if let Some(stream) = &mut self.stream {
            match stream {
                StreamType::Tls(ref mut tls_stream) => match tls_stream.read(message).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
                StreamType::Tcp(ref mut tcp_stream) => match tcp_stream.read(message).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
                StreamType::Unix(ref mut unix_stream) => match unix_stream.read(message).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
            }
        } else {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet.".to_string(),
            ))
        }
    }
    pub async fn read_buf(&mut self, message: &mut Vec<u8>) -> Result<(), tokio::io::Error> {
        if let Some(stream) = &mut self.stream {
            match stream {
                StreamType::Tls(ref mut tls_stream) => match tls_stream.read_buf(message).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
                StreamType::Tcp(ref mut tcp_stream) => match tcp_stream.read_buf(message).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
                StreamType::Unix(ref mut unix_stream) => {
                    match unix_stream.read_buf(message).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    }
                }
            }
        } else {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet.".to_string(),
            ))
        }
    }

    pub async fn read_message(&mut self) -> Result<message::Msg, tokio::io::Error> {
        if let Some(stream) = &mut self.stream {
            match stream {
                StreamType::Tcp(stream) => match stream::read_message(stream).await {
                    Ok(msg) => Ok(msg),
                    Err(e) => {
                        error!("could not read the message from the tcp stream: {}", e);
                        Err(e)
                    }
                },
                StreamType::Tls(stream) => match stream::read_message(stream).await {
                    Ok(msg) => Ok(msg),
                    Err(e) => {
                        error!("could not read the message from the tcp stream: {}", e);
                        Err(e)
                    }
                },
                StreamType::Unix(stream) => match stream::read_message(stream).await {
                    Ok(msg) => Ok(msg),
                    Err(e) => {
                        error!("could not read the message from the tcp stream: {}", e);
                        Err(e)
                    }
                },
            }
        } else {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "client not connected yet.".to_string(),
            ))
        }
    }
}
