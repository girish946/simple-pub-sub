pub mod client_handler;
pub mod message;
pub mod topics;
use log::info;
use std::env;
use std::error::Error;
use tokio::net::TcpListener;

use clap::Parser;

/// Simple pub-sub
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// host name
    #[arg(short, long, default_value = "0.0.0.0")]
    pub host: String,

    /// port
    #[arg(short, long, default_value_t = 6480)]
    pub port: u16,

    /// log_level
    #[arg(short, long, default_value = "trace")]
    pub log_level: Option<String>,
}

pub const LOG_LEVEL: &str = "trace";
pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 6480;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let levels: Vec<String> = vec![
        "trace".to_string(),
        "info".to_string(),
        "debug".to_string(),
        "error".to_string(),
    ];

    let args = Args::parse();
    let addr = format!("{}:{}", args.host, args.port);
    if levels.contains(&args.log_level.clone().unwrap()) {
        env::set_var("RUST_LOG", args.log_level.clone().unwrap());
        env_logger::init();
        info!("using log_level: {}", args.log_level.clone().unwrap());
    }
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
