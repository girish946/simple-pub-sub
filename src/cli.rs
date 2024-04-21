use clap::{Parser, Subcommand};

/// client mode
#[derive(clap::ValueEnum, Clone)]
pub enum ClientType {
    Publish,
    Subscribe,
    Query,
}

/// log leve for the client/server
#[derive(clap::ValueEnum, Clone)]
pub enum LogLevel {
    Trace,
    Warn,
    Info,
    Error,
    Debug,
}

/// the server type subcommand
#[derive(Subcommand)]
pub enum ServerType {
    /// tcp server
    Tcp {
        /// host
        host: String,
        /// port
        port: u16,
    },
    /// unix server
    Unix {
        /// path
        path: String,
    },
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// log level, default: info
    #[clap(long, global = true)]
    pub log_level: Option<LogLevel>,
}

#[derive(Subcommand)]
pub enum Commands {
    Server {
        #[clap(subcommand)]
        server_type: ServerType,
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

        /// socket type
        socket: Option<String>,
    },
}
