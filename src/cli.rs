use clap::{Parser, Subcommand};

/// client mode
#[derive(clap::ValueEnum, Clone)]
pub enum ClientType {
    Publish,
    Subscribe,
    Query,
}

/// log level for the client/server
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

        /// tls certificate
        #[clap(short, long)]
        cert: Option<String>,

        /// tls certificate password
        #[clap(short = 'p', long)]
        cert_password: Option<String>,
    },
    /// unix server
    Unix {
        /// path
        path: String,
    },
}

/// the main command
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// log level, default: info
    #[clap(long, global = true)]
    pub log_level: Option<LogLevel>,
}

/// the subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Server
    Server {
        /// server type, tcp or unix
        #[clap(subcommand)]
        server_type: ServerType,
    },
    /// Client
    Client {
        /// server type, tcp or unix
        #[clap(subcommand)]
        server_tyepe: ServerType,
        /// client mode
        client_type: ClientType,
        /// topic to publish/subscribe
        topic: String,
        /// message to be published
        message: Option<String>,
    },
}
