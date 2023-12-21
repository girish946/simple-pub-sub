mod client_handler;
use crate::topics;
use log::info;
use tokio::net::TcpListener;

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
