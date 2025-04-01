pub mod cli;
use crate::cli::{Cli, ClientType, Commands, LogLevel, ServerType};
use clap::{Parser, ValueEnum};
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

    let queue_capacity = cli.capacity.unwrap_or(1024);

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
                    capacity: queue_capacity,
                };
                match server.start().await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{:?}", e);
                    }
                };
            }
            ServerType::Unix { path } => {
                let server = server::Unix {
                    path: path.clone(),
                    capacity: queue_capacity,
                };
                match server.start().await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{:?}", e);
                    }
                };
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
                info!("Connecting to: {addr}");
                client_ = client::PubSubClient::Tcp(client::PubSubTcpClient {
                    server: server.clone(),
                    port,
                    cert,
                    cert_password,
                });
            } else {
                return Err("Socket type not supported".into());
            }

            let mut client = client::Client::new(client_);
            match client.connect().await {
                Ok(()) => {}
                Err(e) => {
                    error!("{:?}", e);

                    return Ok(());
                }
            };

            //client.on_message(client::on_message);

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

                    let callback_fn = |topic: String, message: &[u8]| {
                        let msg_str = String::from_utf8(message.to_vec()).unwrap_or("".to_string());
                        println!("{}: {}", topic, msg_str);
                    };
                    let _ = client.subscribe(topic.clone(), Box::new(callback_fn)).await;
                }
                ClientType::Query => {
                    info!("Querying topic '{}'", topic);
                    match client.query(topic.clone()).await {
                        Ok(resp) => {
                            info!("{}", resp)
                        }
                        Err(e) => {
                            error!("{:?}", e)
                        }
                    };
                }
            }
        }
        Commands::Completion { shell } => {
            completion(shell);
        }
    }
    Ok(())
}

fn completion(shell: &str) {
    use clap::CommandFactory;
    let mut cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Default::default();
    match man.render(&mut buffer) {
        Ok(_) => {}
        Err(e) => {
            println!("Error while generating the completions:{}", e);
        }
    };

    let shell = match clap_complete::Shell::from_str(&shell, true) {
        Ok(shell) => shell,
        Err(_) => {
            eprintln!("Shell not supported {}", shell);
            return;
        }
    };
    let bin_name = "simple-pub-sub";
    clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
}
