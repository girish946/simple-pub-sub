mod client_handler;
use crate::topics;
use anyhow::Result;
use log::info;
use std::fs::File;
use std::io::Read;
use tokio::net::TcpListener;
use tokio::net::UnixListener;
use tokio_native_tls::native_tls::{Identity, TlsAcceptor};

pub trait ServerTrait {
    fn start(&self) -> impl std::future::Future<Output = Result<()>> + Send;
}
pub struct Tcp {
    pub host: String,
    pub port: u16,
    pub cert: Option<String>,
    pub cert_password: Option<String>,
    pub capacity: usize,
}

impl ServerTrait for Tcp {
    /// Starts the simple pub sub server for the given server type
    /// ```
    /// use simple_pub_sub::server::ServerTrait as _;
    /// // for tcp
    /// async fn run_server_tcp(){
    ///   let server = simple_pub_sub::server::ServerType::Tcp(simple_pub_sub::server::Tcp {
    ///     host: "localhost".to_string(),
    ///     port: 6480,
    ///     cert: None,
    ///     cert_password: None,
    ///     capacity: 1024,
    ///   });
    ///   let _ = server.start().await;
    /// }
    /// // for tls
    /// async fn run_server_tls(){
    ///   let server = simple_pub_sub::server::ServerType::Tcp(simple_pub_sub::server::Tcp {
    ///     host: "localhost".to_string(),
    ///     port: 6480,
    ///     cert: Some("certs/cert.pem".to_string()),
    ///     cert_password: Some("password".to_string()),
    ///     capacity: 1024,
    ///   });
    ///   let _ = server.start().await;
    /// }
    /// ```
    async fn start(&self) -> Result<()> {
        if let Some(cert) = &self.cert {
            start_tls_server(
                self.host.clone(),
                self.port,
                cert.clone(),
                self.cert_password.clone(),
                self.capacity,
            )
            .await
        } else {
            start_tcp_server(format!("{}:{}", self.host, self.port), self.capacity).await
        }
    }
}
pub struct Unix {
    pub path: String,
    pub capacity: usize,
}

impl ServerTrait for Unix {
    /// start the pub-sub server over a unix socket
    ///```
    /// use crate::simple_pub_sub::server::ServerTrait as _;
    /// let server = simple_pub_sub::server::ServerType::Unix(simple_pub_sub::server::Unix {
    ///   path: "/tmp/sample.sock".to_string(),
    ///   capacity: 1024,
    /// });
    /// let result = server.start();
    ///```
    async fn start(&self) -> Result<()> {
        start_unix_server(self.path.clone(), self.capacity).await
    }
}
impl Drop for Unix {
    fn drop(&mut self) {
        if std::path::Path::new(&self.path).exists() {
            std::fs::remove_file(&self.path).unwrap();
        }
    }
}

pub enum ServerType {
    Tcp(Tcp),
    Unix(Unix),
}
impl ServerTrait for ServerType {
    /// starts the simple-pub-sub server on the given server type
    ///```
    /// use simple_pub_sub::server::ServerTrait as _;
    ///
    /// // for tcp
    ///   let server = simple_pub_sub::server::ServerType::Tcp(simple_pub_sub::server::Tcp {
    ///     host: "localhost".to_string(),
    ///     port: 6480,
    ///     cert: None,
    ///     cert_password: None,
    ///     capacity: 1024,
    ///   });
    ///   server.start();
    ///
    /// // for tls
    ///
    ///   let server = simple_pub_sub::server::ServerType::Tcp(simple_pub_sub::server::Tcp {
    ///     host: "localhost".to_string(),
    ///     port: 6480,
    ///     cert: Some("certs/cert.pem".to_string()),
    ///     cert_password: Some("password".to_string()),
    ///     capacity: 1024,
    ///   });
    ///   server.start();
    ///
    /// // for unix socket
    /// use crate::simple_pub_sub::server::ServerTrait as _;
    /// let server = simple_pub_sub::server::ServerType::Unix(simple_pub_sub::server::Unix {
    ///   path: "/tmp/sample.sock".to_string(),
    ///   capacity: 1024,
    /// });
    /// let result = server.start();
    ///```
    async fn start(&self) -> Result<()> {
        match self {
            ServerType::Tcp(tcp) => tcp.start().await,
            ServerType::Unix(unix) => unix.start().await,
        }
    }
}

pub struct Server {
    pub server_type: ServerType,
}

impl Server {
    pub async fn start(&self) -> Result<()> {
        self.server_type.start().await
    }
}

/// Started a tls server on the given address with the given certificate (.pfx file)
async fn start_tls_server(
    host: String,
    port: u16,
    cert: String,
    cert_password: Option<String>,
    capacity: usize,
) -> Result<()> {
    // Load TLS identity (certificate and private key)
    let mut file = File::open(&cert)?;
    let mut identity_vec = vec![];
    file.read_to_end(&mut identity_vec)?;

    let identity: Identity;
    if let Some(cert_password) = cert_password {
        identity = Identity::from_pkcs12(&identity_vec, cert_password.as_str())?;
    } else {
        identity = Identity::from_pkcs12(&identity_vec, "")?;
    }

    let acceptor = TlsAcceptor::builder(identity).build()?;
    let acceptor = tokio_native_tls::TlsAcceptor::from(acceptor);

    // Bind TCP listener
    let listener = TcpListener::bind(format!("{host}:{port}")).await?;

    println!("Server listening on port 4433");
    let tx = topics::get_global_broadcaster(capacity);
    let _topic_handler = tokio::spawn(topics::topic_manager(tx.clone()));
    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from {:?}", addr);
        let acceptor = acceptor.clone();
        let tls_stream = acceptor.accept(stream).await?;
        client_handler::handle_client(tls_stream, tx.clone()).await;
    }
}

/// Starts a tcp server on the given address
async fn start_tcp_server(addr: String, capacity: usize) -> Result<()> {
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);
    info!("Getting global broadcaster");

    let tx = topics::get_global_broadcaster(capacity);
    let _topic_handler = tokio::spawn(topics::topic_manager(tx.clone()));
    loop {
        let (socket, addr) = listener.accept().await?;
        info!("Addr is: {addr}");
        client_handler::handle_client(socket, tx.clone()).await;
    }
}

/// Starts a unix server on the given path
async fn start_unix_server(path: String, capacity: usize) -> Result<()> {
    if std::path::Path::new(&path).exists() {
        std::fs::remove_file(path.clone())?;
    }

    let listener = UnixListener::bind(&path)?;
    info!("Listening on: {}", path);
    info!("Getting global broadcaster");
    let tx = topics::get_global_broadcaster(capacity);
    let _topic_handler = tokio::spawn(topics::topic_manager(tx.clone()));
    loop {
        let (socket, addr) = listener.accept().await?;
        info!("Addr is: {:?}", addr.as_pathname());
        client_handler::handle_client(socket, tx.clone()).await;
    }
}
