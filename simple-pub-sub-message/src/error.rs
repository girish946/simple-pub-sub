use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeaderError {
    /// invalid header buffer length
    #[error("Invalid header buffer length")]
    InvalidHeaderBufferLength,
    /// invalid header value or padding
    #[error("Invalid header value or padding")]
    InvalidHeadOrTail,
    /// unsupported version of the packet
    #[error("Unsupported version of the packet")]
    UnsupportedVersion,
    /// invalid packet type
    #[error("Invalid packet type")]
    InvalidPacketType,
    /// invalid topic length
    #[error("Invalid topic length")]
    InvalidTopicLength,
    /// invalid message length
    #[error("Invalid message length: `{0}`")]
    InvalidMessageLength(usize),
    /// invalid request/response type
    #[error("Invalid request/response type")]
    InvalidResponseType,
}
