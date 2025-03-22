use crate::message;
use crate::message::Msg;
use crate::stream;
use crate::Header;
use crate::PktType;
use anyhow::bail;
use anyhow::Result;
use log::{info, trace};
use std::fs::File;
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

impl StreamType {
    async fn read_message(&mut self) -> Result<Msg> {
        match self {
            StreamType::Tcp(stream) => Ok(stream::read_message(stream).await?),
            StreamType::Tls(stream) => Ok(stream::read_message(stream).await?),
            StreamType::Unix(stream) => Ok(stream::read_message(stream).await?),
        }
    }

    async fn read_buf(&mut self, message: &mut Vec<u8>) -> Result<usize> {
        let size = match self {
            StreamType::Tls(ref mut tls_stream) => tls_stream.read_buf(message).await?,
            StreamType::Tcp(ref mut tcp_stream) => tcp_stream.read_buf(message).await?,
            StreamType::Unix(ref mut unix_stream) => unix_stream.read_buf(message).await?,
        };
        Ok(size)
    }

    async fn read(&mut self, message: &mut [u8]) -> Result<usize> {
        let size = match self {
            StreamType::Tls(ref mut tls_stream) => tls_stream.read(message).await?,
            StreamType::Tcp(ref mut tcp_stream) => tcp_stream.read(message).await?,
            StreamType::Unix(ref mut unix_stream) => unix_stream.read(message).await?,
        };
        Ok(size)
    }

    async fn write_all(&mut self, message: Vec<u8>) -> Result<()> {
        match self {
            StreamType::Tls(tls_stream) => tls_stream.write_all(&message).await?,
            StreamType::Tcp(ref mut tcp_stream) => tcp_stream.write_all(&message).await?,
            StreamType::Unix(ref mut unix_stream) => unix_stream.write_all(&message).await?,
        };
        Ok(())
    }
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

    async fn connect_tls(&mut self, url: String, cert: String) -> Result<()> {
        // Load CA certificate
        let mut file = File::open(cert)?;
        let mut ca_cert = vec![];
        file.read_to_end(&mut ca_cert)?;
        let ca_cert = Certificate::from_pem(&ca_cert)?;

        // Configure TLS
        let connector = TlsConnector::builder()
            .add_root_certificate(ca_cert)
            .build()?;

        let connector = tokio_native_tls::TlsConnector::from(connector);

        // Connect to the server
        let stream = TcpStream::connect(url).await?;

        // create the StreamType::Tls
        self.stream = Some(StreamType::Tls(
            connector.connect("localhost", stream).await?,
        ));
        Ok(())
    }

    /// Connects to the server
    pub async fn connect(&mut self) -> Result<()> {
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
    pub async fn post(&mut self, msg: Msg) -> Result<Vec<u8>> {
        self.write(msg.bytes()).await?;
        let mut response_message_buffer: Vec<u8>;
        response_message_buffer = vec![0; 8];
        self.read(&mut response_message_buffer).await?;
        trace!("buf: {:?}", response_message_buffer);
        let response_header = Header::try_from(response_message_buffer.clone())?;
        trace!("resp: {:?}", response_header);
        let mut response_body = Vec::with_capacity(response_header.message_length as usize);
        trace!("reading remaining bytes");
        self.read_buf(&mut response_body).await?;
        response_message_buffer.extend(response_body);
        Ok(response_message_buffer)
    }

    /// Publishes the message to the given topic
    pub async fn publish(&mut self, topic: String, message: Vec<u8>) -> Result<()> {
        let msg: Msg = Msg::new(PktType::PUBLISH, topic, Some(message));
        trace!("msg: {:?}", msg);
        let buf = self.post(msg).await?;
        trace!("the raw buffer is: {:?}", buf);
        let resp_: Header = Header::try_from(buf)?;
        trace!("{:?}", resp_);
        Ok(())
    }

    /// Sends the query message to the server
    pub async fn query(&mut self, topic: String) -> Result<String> {
        let msg: Msg = Msg::new(
            PktType::QUERY,
            topic,
            Some(" ".to_string().as_bytes().to_vec()),
        );
        trace!("msg: {:?}", msg);

        self.write(msg.bytes()).await?;
        let msg = self.read_message().await?;
        Ok(String::from_utf8(msg.message)?)
    }

    /// subscribes to the given topic
    pub async fn subscribe(&mut self, topic: String) -> Result<()> {
        let msg: message::Msg = message::Msg::new(PktType::SUBSCRIBE, topic, None);
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
            bail!("no callback function is set");
        }
    }

    pub async fn write(&mut self, message: Vec<u8>) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            stream.write_all(message).await?;
            Ok(())
        } else {
            bail!("client is not connected yet");
        }
    }

    pub async fn read(&mut self, message: &mut [u8]) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            let size = stream.read(message).await?;
            trace!("read: {} bytes", size);
            Ok(())
        } else {
            bail!("client is not connected yet");
        }
    }
    pub async fn read_buf(&mut self, message: &mut Vec<u8>) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            let size = stream.read_buf(message).await?;
            trace!("read: {} bytes", size);
            Ok(())
        } else {
            bail!("client is not connected yet");
        }
    }

    pub async fn read_message(&mut self) -> Result<message::Msg> {
        if let Some(stream) = &mut self.stream {
            stream.read_message().await
        } else {
            bail!("client is not connected yet");
        }
    }
}
