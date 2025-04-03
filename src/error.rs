use thiserror::Error;

#[derive(Error, Debug)]
pub enum PubSubError {
    /// client is not connected yet
    #[error("Client is not connected")]
    ClientNotConnected,
}
