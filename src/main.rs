pub mod cli;
use crate::cli::{Cli, ClientType, Commands, LogLevel, ServerType};
use clap::Parser;
use log::{error, info};
use simple_pub_sub::server::ServerTrait as _;
use simple_pub_sub::{client, server};
use std::env;
use std::error::Error;

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
        Commands::Server { server_type } => match server_type {
            ServerType::Tcp {
                host,
                port,
                cert,
                cert_password,
            } => {
                let server = server::Tcp {
                    host: host.to_string(),
                    port: *port,
                    cert: cert.clone(),
                    cert_password: cert_password.clone(),
                };
                let result = server.start().await;
                info!("{:?}", result);
            }
            ServerType::Unix { path } => {
                let result = server::Unix { path: path.clone() }.start().await;
                info!("{:?}", result);
            }
        },
        Commands::Client {
            client_type,
            topic,
            message,
            server_tyepe,
        } => {
            let (server, port, socket, cert, cert_password): (
                &String,
                Option<&u16>,
                String,
                Option<String>,
                Option<String>,
            ) = match server_tyepe {
                ServerType::Tcp {
                    host,
                    port,
                    cert,
                    cert_password,
                } => (
                    host,
                    Some(port),
                    "tcp".to_string(),
                    cert.clone(),
                    cert_password.clone(),
                ),
                ServerType::Unix { path } => (path, Some(&0), "unix".to_string(), None, None),
            };

            let client_: client::PubSubClient;

            if socket == "unix" {
                client_ = client::PubSubClient::Unix(client::PubSubUnixClient {
                    path: server.clone(),
                });
            } else if socket == "tcp" {
                let port = match port {
                    Some(port) => *port,
                    None => 6480,
                };
                let addr = format!("{server}:{port}");
                info!("connecting to: {addr}");
                client_ = client::PubSubClient::Tcp(client::PubSubTcpClient {
                    server: server.clone(),
                    port,
                    cert,
                    cert_password,
                });
            } else {
                return Err("socket type not supported".into());
            }

            let mut client = client::Client::new(client_);
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
