pub mod client_handler;
pub mod message;
pub mod topics;
use clap::{Parser, Subcommand};
use log::info;
use std::env;
use std::error::Error;
use tokio::net::TcpListener;

/// client mode
#[derive(clap::ValueEnum, Clone)]
enum ClientType {
    Publish,
    Subscribe,
    Query,
}

/// log leve for the client/server
#[derive(clap::ValueEnum, Clone)]
enum LogLevel {
    Teace,
    Warn,
    Info,
    Error,
    Debug,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// log level, default: info
    #[clap(long, global = true)]
    log_level: Option<LogLevel>,
}

#[derive(Subcommand)]
enum Commands {
    Server {
        /// hoststring
        host: String,
        /// port
        port: u16,
    },
    Client {
        /// server
        server: String,
        /// server port
        port: u16,
        /// client mode
        client_type: ClientType,
        /// topic to publish/subscribe
        topic: String,
        /// message to be published
        message: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let log_level: &str = match cli.log_level {
        Some(LogLevel::Teace) => "trace",
        Some(LogLevel::Warn) => "warn",
        Some(LogLevel::Info) => "info",
        Some(LogLevel::Error) => "error",
        Some(LogLevel::Debug) => "debug",
        None => "info",
    };
    env::set_var("RUST_LOG", log_level);
    env_logger::init();

    match &cli.command {
        Commands::Server { host, port } => {
            let addr = format!("{}:{}", host, port);
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
        Commands::Client {
            server,
            port,
            client_type,
            topic,
            message,
        } => {
            let addr = format!("{server}:{port}");
            info!("connecting to: {addr}");
            match client_type {
                ClientType::Publish => {
                    info!(
                        "Publishing message '{}' on topic '{}'",
                        message.as_deref().unwrap_or(""),
                        topic
                    );
                    // Publish logic here
                }
                ClientType::Subscribe => {
                    info!("Subscribing to topic '{}'", topic);
                    // Subscribe logic here
                }
                ClientType::Query => {
                    info!("Querying topic '{}'", topic);
                    // Query logic here
                }
            }
        }
    }
    Ok(())
}
