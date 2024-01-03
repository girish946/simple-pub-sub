use clap::{Parser, Subcommand};
use log::{error, info};
use simple_pub_sub::{client, server};
use std::env;
use std::error::Error;

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
    Trace,
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
        Some(LogLevel::Trace) => "trace",
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
            let _ = server::start_server(addr).await;
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

            let mut client = client::Client::new(server.clone(), port.clone());
            match client.connect().await {
                Ok(()) => {}
                Err(e) => {
                    error!("{}", e.to_string());
                    return Ok(());
                }
            };

            client.on_message(client::on_message);

            match client_type {
                ClientType::Publish => {
                    info!(
                        "Publishing message '{}' on topic '{}'",
                        message.as_deref().unwrap_or(""),
                        topic
                    );
                    let msg = match message {
                        Some(msg) => msg.as_bytes().to_vec(),
                        None => vec![],
                    };
                    let _ = client.publish(topic.clone(), msg).await;
                }
                ClientType::Subscribe => {
                    info!("Subscribing to topic '{}'", topic);
                    let _ = client.subscribe(topic.clone()).await;
                }
                ClientType::Query => {
                    info!("Querying topic '{}'", topic);
                    match client.query(topic.clone()).await {
                        Ok(resp) => {
                            info!("{}", resp)
                        }
                        Err(e) => {
                            error!("{}", e.to_string())
                        }
                    };
                }
            }
        }
    }
    Ok(())
}
