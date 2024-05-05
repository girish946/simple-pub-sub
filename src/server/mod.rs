mod client_handler;
use crate::topics;
use log::info;
use tokio::net::TcpListener;
use tokio::net::UnixListener;

pub async fn start_server(addr: String) -> Result<(), tokio::io::Error> {
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);
    info!("getting global broadcaster");

    let tx = topics::get_global_broadcaster();

    let _topic_handler = tokio::spawn(topics::topic_manager(tx.clone()));

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("addr is: {addr}");
        client_handler::handle_clinet(socket, tx.clone()).await;
    }
}

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
        client_handler::handle_clinet(socket, tx.clone()).await;
    }
}
