pub mod client_handler;
pub mod message;
use log::info;
use std::env;
use std::error::Error;
use tokio::net::TcpListener;

pub const LOG_LEVEL: &str = "trace";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", LOG_LEVEL);
    env_logger::init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6480".to_string());

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("addr is: {addr}");
        client_handler::handle_clinet(socket).await;
    }
}
