mod client_handler;
use crate::topics;
use log::error;
use log::info;
use std::fs::File;
use std::io::Read;
use tokio::net::TcpListener;
use tokio::net::UnixListener;
use tokio_native_tls::native_tls::{Identity, TlsAcceptor};

/// Started a tls server on the given address with the given certificate (.pfx file)
pub async fn start_tls_server(
    host: String,
    port: u16,
    cert: String,
    cert_password: Option<String>,
) -> Result<(), tokio::io::Error> {
    // Load TLS identity (certificate and private key)

    let mut file = match File::open(&cert) {
        Ok(file) => file,
        Err(e) => {
            error!("could not open identity file: {}: {}", cert, e);
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "could not open identity file",
            ));
        }
    };
    let mut identity_vec = vec![];
    file.read_to_end(&mut identity_vec)?;

    let identity: Identity;
    if let Some(cert_password) = cert_password {
        identity = match Identity::from_pkcs12(&identity_vec, cert_password.as_str()) {
            Ok(identity) => identity,
            Err(e) => {
                error!("could not parse identity file: {}", e);
                return Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, e));
            }
        };
    } else {
        identity = match Identity::from_pkcs12(&identity_vec, "") {
            Ok(identity) => identity,
            Err(e) => {
                error!("could not parse identity file: {}", e);
                return Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, e));
            }
        };
    }

    let acceptor = TlsAcceptor::builder(identity)
        .build()
        .expect("cannot create TLS acceptor");
    let acceptor = tokio_native_tls::TlsAcceptor::from(acceptor);

    // Bind TCP listener
    let listener = TcpListener::bind(format!("{host}:{port}"))
        .await
        .expect("cannot bind to address");

    println!("Server listening on port 4433");
    let tx = topics::get_global_broadcaster();

    let _topic_handler = tokio::spawn(topics::topic_manager(tx.clone()));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from {:?}", addr);
        let acceptor = acceptor.clone();
        let tls_stream = match acceptor.accept(stream).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("could not accept TLS connection: {}", e);
                continue;
            }
        };
        client_handler::handle_client(tls_stream, tx.clone()).await;
    }
}

/// Starts a tcp server on the given address
pub async fn start_tcp_server(addr: String) -> Result<(), tokio::io::Error> {
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);
    info!("getting global broadcaster");

    let tx = topics::get_global_broadcaster();

    let _topic_handler = tokio::spawn(topics::topic_manager(tx.clone()));

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("addr is: {addr}");
        client_handler::handle_client(socket, tx.clone()).await;
    }
}

/// Starts a unix server on the given path
pub async fn start_unix_server(path: String) -> Result<(), tokio::io::Error> {
    if std::path::Path::new(&path).exists() {
        std::fs::remove_file(path.clone())?;
    }

    let listener = UnixListener::bind(&path)?;
    info!("Listening on: {}", path);
    info!("getting global broadcaster");
    let tx = topics::get_global_broadcaster();
    let _topic_handler = tokio::spawn(topics::topic_manager(tx.clone()));
    loop {
        let (socket, addr) = listener.accept().await?;
        info!("addr is: {:?}", addr.as_pathname());
        client_handler::handle_client(socket, tx.clone()).await;
    }
}
