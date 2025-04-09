use crate::error::PubSubError::ClientNotConnected;
use crate::message;
use crate::message::Msg;
use crate::stream;
use crate::Header;
use crate::PktType;
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
    /// domain/host for the server
    /// for example: `127.0.0.1`
    pub server: String,
    /// port for the simple_pub_sub server
    pub port: u16,
    /// tls certificate (`.pem`) file
    pub cert: Option<String>,
    /// password for the tls certificate
    pub cert_password: Option<String>,
}

/// Simple pub sub Client for Unix connection
#[derive(Debug, Clone)]
pub struct PubSubUnixClient {
    /// path for the unix sock file
    /// for example: `/tmp/simple-pub-sub.sock`
    pub path: String,
}

/// Simple pub sub Client
#[derive(Debug, Clone)]
pub enum PubSubClient {
    /// tcp client for the simple pub sub
    Tcp(PubSubTcpClient),
    /// unix socket client for the simple pub sub
    Unix(PubSubUnixClient),
}

impl PubSubClient {
    fn server(&self) -> &str {
        match self {
            PubSubClient::Tcp(pub_sub_tcp_client) => &pub_sub_tcp_client.server,
            PubSubClient::Unix(pub_sub_unix_client) => &pub_sub_unix_client.path,
        }
    }
}

/// Stream for Tcp and Unix connection
#[derive(Debug)]
pub enum StreamType {
    /// tcp stream
    Tcp(TcpStream),
    /// tls stream
    Tls(Box<TlsStream<TcpStream>>),
    /// unix socket stream
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

/// Simple pub sub Client
#[derive(Debug)]
pub struct Client {
    pub client_type: PubSubClient,
    stream: Option<StreamType>,
}

/// default implementation for callback function
pub fn on_message(topic: String, message: Vec<u8>) {
    match String::from_utf8(message.clone()) {
        Ok(msg_str) => {
            info!("Topic: {} message: {}", topic, msg_str);
        }
        Err(_) => {
            info!("Topic: {} message: {:?}", topic, message);
        }
    };
}

impl Client {
    /// Creates a new instance of `Client`
    /// ```
    /// use simple_pub_sub::client::{self, PubSubClient, Client};
    /// let client_type = simple_pub_sub::client::PubSubTcpClient {
    ///        server: "localhost".to_string(),
    ///        port: 6480,
    ///        cert: None,
    ///        cert_password: None,
    /// };
    ///
    /// // initialize the client.
    /// let mut pub_sub_client = simple_pub_sub::client::Client::new(
    ///     simple_pub_sub::client::PubSubClient::Tcp(client_type)
    /// );
    /// ```
    pub fn new(client_type: PubSubClient) -> Self {
        Client {
            client_type,
            stream: None,
        }
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
        let stream = TcpStream::connect(&url).await?;

        // create the StreamType::Tls
        let connector = connector.connect(self.client_type.server(), stream).await?;
        self.stream = Some(StreamType::Tls(Box::new(connector)));
        Ok(())
    }

    /// Connects to the server
    ///```
    /// use simple_pub_sub::client::{self, PubSubClient, Client};
    /// let client_type = simple_pub_sub::client::PubSubTcpClient {
    ///        server: "localhost".to_string(),
    ///        port: 6480,
    ///        cert: None,
    ///        cert_password: None,
    /// };
    ///
    /// // initialize the client.
    /// let mut pub_sub_client = simple_pub_sub::client::Client::new(
    ///     simple_pub_sub::client::PubSubClient::Tcp(client_type),
    /// );
    /// pub_sub_client.connect();
    /// ```
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
    ///```
    /// use simple_pub_sub::client::{PubSubClient, Client};
    /// use simple_pub_sub::message::Msg;
    /// use simple_pub_sub::PktType;
    /// async fn publish_msg(){
    ///   let client_type = simple_pub_sub::client::PubSubTcpClient {
    ///          server: "localhost".to_string(),
    ///          port: 6480,
    ///          cert: None,
    ///          cert_password: None,
    ///   };
    ///
    /// // initialize the client.
    /// let mut pub_sub_client = simple_pub_sub::client::Client::new(
    ///     simple_pub_sub::client::PubSubClient::Tcp(client_type),
    /// );
    /// pub_sub_client.connect().await.unwrap();
    /// let msg = Msg::new(PktType::PUBLISH, "Test".to_string(), Some(b"The message".to_vec()));
    ///   pub_sub_client.post(msg).await.unwrap();
    /// }
    /// ```
    pub async fn post(&mut self, msg: Msg) -> Result<Vec<u8>> {
        self.write(msg.bytes()).await?;
        let mut response_message_buffer: Vec<u8>;
        response_message_buffer = vec![0; 8];
        self.read(&mut response_message_buffer).await?;
        trace!("Buf: {:?}", response_message_buffer);
        let response_header = Header::try_from(response_message_buffer.clone())?;
        trace!("Resp: {:?}", response_header);
        let mut response_body = Vec::with_capacity(response_header.message_length as usize);
        trace!("Reading remaining bytes");
        self.read_buf(&mut response_body).await?;
        response_message_buffer.extend(response_body);
        Ok(response_message_buffer)
    }

    /// Publishes the message to the given topic
    /// ```
    /// use simple_pub_sub::client::{PubSubClient, Client};
    /// async fn publish_msg(){
    ///   let client_type = simple_pub_sub::client::PubSubTcpClient {
    ///          server: "localhost".to_string(),
    ///          port: 6480,
    ///          cert: None,
    ///          cert_password: None,
    ///   };
    ///
    /// // initialize the client.
    /// let mut pub_sub_client = simple_pub_sub::client::Client::new(
    ///     simple_pub_sub::client::PubSubClient::Tcp(client_type),
    /// );
    /// pub_sub_client.connect().await.unwrap();
    /// // subscribe to the given topic.
    /// pub_sub_client
    ///   .publish(
    ///     "Abc".to_string(),
    ///     "Test message".to_string().into_bytes().to_vec(),
    ///   ).await.unwrap();
    /// }
    /// ```
    pub async fn publish(&mut self, topic: String, message: Vec<u8>) -> Result<()> {
        let msg: Msg = Msg::new(PktType::PUBLISH, topic, Some(message));
        trace!("Msg: {:?}", msg);
        let buf = self.post(msg).await?;
        trace!("The raw buffer is: {:?}", buf);
        let resp_: Header = Header::try_from(buf)?;
        trace!("{:?}", resp_);
        Ok(())
    }

    /// Sends the query message to the server
    /// ```
    /// use simple_pub_sub::client::{self, PubSubClient, Client};
    /// async fn query(){
    ///   let client_type = simple_pub_sub::client::PubSubTcpClient {
    ///          server: "localhost".to_string(),
    ///          port: 6480,
    ///          cert: None,
    ///          cert_password: None,
    ///   };
    ///
    /// // initialize the client.
    /// let mut pub_sub_client = simple_pub_sub::client::Client::new(
    ///     simple_pub_sub::client::PubSubClient::Tcp(client_type),
    /// );
    /// pub_sub_client.connect().await.unwrap();
    /// pub_sub_client.query("Test".to_string());
    /// }
    /// ```
    pub async fn query(&mut self, topic: String) -> Result<String> {
        let msg: Msg = Msg::new(
            PktType::QUERY,
            topic,
            Some(" ".to_string().as_bytes().to_vec()),
        );
        trace!("Msg: {:?}", msg);

        self.write(msg.bytes()).await?;
        let msg = self.read_message().await?;
        Ok(String::from_utf8(msg.message)?)
    }

    /// subscribes to the given topic
    ///```
    /// use simple_pub_sub::client::{self, PubSubClient, Client};
    /// let client_type = simple_pub_sub::client::PubSubTcpClient {
    ///        server: "localhost".to_string(),
    ///        port: 6480,
    ///        cert: None,
    ///        cert_password: None,
    /// };
    /// // initialize the client.
    /// let mut pub_sub_client = simple_pub_sub::client::Client::new(
    ///     simple_pub_sub::client::PubSubClient::Tcp(client_type));
    /// pub_sub_client.subscribe("Test".to_string());
    /// pub_sub_client.run();
    /// ```
    pub async fn subscribe(&mut self, topic: String) -> Result<()> {
        let msg: message::Msg = message::Msg::new(PktType::SUBSCRIBE, topic, None);
        trace!("Msg: {:?}", msg);
        self.write(msg.bytes()).await
    }

    async fn write(&mut self, message: Vec<u8>) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            stream.write_all(message).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!(ClientNotConnected))
        }
    }

    async fn read(&mut self, message: &mut [u8]) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            let size = stream.read(message).await?;
            trace!("Read: {} bytes", size);
            Ok(())
        } else {
            Err(anyhow::anyhow!(ClientNotConnected))
        }
    }
    async fn read_buf(&mut self, message: &mut Vec<u8>) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            let size = stream.read_buf(message).await?;
            trace!("Read: {} bytes", size);
            Ok(())
        } else {
            Err(anyhow::anyhow!(ClientNotConnected))
        }
    }

    /// reads the incoming message from the server
    /// useful when you need to read the messages in loop
    /// ```
    /// use simple_pub_sub::client::{self, PubSubClient, Client};
    ///
    /// async fn read_messages(){
    ///   let client_type = simple_pub_sub::client::PubSubTcpClient {
    ///          server: "localhost".to_string(),
    ///          port: 6480,
    ///          cert: None,
    ///          cert_password: None,
    ///   };
    ///   // initialize the client.
    ///   let mut pub_sub_client = simple_pub_sub::client::Client::new(
    ///       simple_pub_sub::client::PubSubClient::Tcp(client_type));
    ///   pub_sub_client.connect().await?;
    ///   pub_sub_client.subscribe("Test".to_string()).await?;
    ///
    ///   loop {
    ///       match pub_sub_client.read_message().await{
    ///           Ok(msg)=>{
    ///               println!("{}: {:?}", msg.topic, msg.message);
    ///           }
    ///           Err(e)=>{
    ///               println!("error: {:?}", e);
    ///               break
    ///           }
    ///       }
    ///   }
    /// }
    /// ```
    pub async fn read_message(&mut self) -> Result<Msg> {
        if let Some(stream) = &mut self.stream {
            stream.read_message().await
        } else {
            Err(anyhow::anyhow!(ClientNotConnected))
        }
    }
}
