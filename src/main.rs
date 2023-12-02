pub mod client_handler;
pub mod message;
pub mod topics;
use log::info;
use std::env;
use std::error::Error;
use tokio::net::TcpListener;

pub const LOG_LEVEL: &str = "trace";
pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 6480;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", LOG_LEVEL);
    env_logger::init();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT));

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
