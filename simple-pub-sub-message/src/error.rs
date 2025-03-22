use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeaderError {
    /// invalid header buffer length
    #[error("invalid header buffer length")]
    InvalidHeaderBufferLength,
    /// invalid header value or padding
    #[error("invalid header value or padding")]
    InvalidHeadOrTail,
    /// unsupported version of the packet
    #[error("unsupported version of the packet")]
    UnsupportedVersion,
    /// invalid packet type
    #[error("invalid packet type")]
    InvalidPacketType,
    /// invalid topic length
    #[error("invalid topic length")]
    InvalidTopicLength,
    /// invalid message length
    #[error("invalid message length: `{0}`")]
    InvalidMessageLength(usize),
    /// invalid request/response type
    #[error("invalid request/response type")]
    InvalidResuestResponseType,
}
