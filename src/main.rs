pub mod cli;
use crate::cli::{Cli, ClientType, Commands, LogLevel, ServerType};
use clap::Parser;
use log::{error, info};
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
            ServerType::Tcp { host, port } => {
                let addr = format!("{}:{}", host, port);
                let _ = server::start_server(addr).await;
            }
            ServerType::Unix { path } => {
                let _ = server::start_unix_server(path.clone()).await;
            }
        },
        // Commands::Server { host, port } => {

        //     let addr = format!("{}:{}", host, port);
        //     let _ = server::start_server(addr).await;
        // }
        Commands::Client {
            server,
            port,
            client_type,
            topic,
            message,
            socket,
        } => {
            let addr = format!("{server}:{port}");
            info!("connecting to: {addr}");
            let client_: client::PubSubClient;
            match socket {
                Some(socket) => {
                    if socket == "unix" {
                        client_ = client::PubSubClient::Unix(client::PubSubUnixClient {
                            path: server.clone(),
                        });
                    } else if socket == "tcp" {
                        client_ = client::PubSubClient::Tcp(client::PubSubTcpClient {
                            server: server.clone(),
                            port: *port,
                        });
                    } else {
                        return Err("socket type not supported".into());
                    }
                }
                None => {
                    client_ = client::PubSubClient::Tcp(client::PubSubTcpClient {
                        server: "0.0.0.0".to_string(),
                        port: 6480,
                    });
                }
            };

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
