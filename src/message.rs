use log::{error, trace};
pub use simple_pub_sub_message::header::Header;
pub use simple_pub_sub_message::pkt::PktType;
use tokio::sync::broadcast::Sender;

/// Packet Type Publish
pub const PUBLISH: u8 = 0x02;
/// Packet Type Subscribe
pub const SUBSCRIBE: u8 = 0x03;
/// Packet Type Unsubscribe
pub const UNSUBSCRIBE: u8 = 0x04;
/// Packet Type Query
pub const QUERY: u8 = 0x05;
/// Packet Type Publish Acknowledgement
pub const PUBLISHACK: u8 = 0x0B;
/// Packet Type Subscribe Acknowledgement
pub const SUBSCRIBEACK: u8 = 0x0C;
/// Packet Type Unsubscribe Acknowledgement
pub const UNSUBSCRIBEACK: u8 = 0x0D;
/// Packet Type Query Response
pub const QUERYRESP: u8 = 0x0E;

/// supported versions for the pub-sub header format/protocol.
pub const SUPPORTED_VERSIONS: [[u8; 2]; 1] = [[0x00, 0x01]];

/// default version for the pub-sub header format/protocol.
pub const DEFAULT_VERSION: [u8; 2] = [0x00, 0x01];

/// error types for the `Header`.
#[derive(Debug, Clone)]
pub enum HeaderError {
    /// invalid header buffer length
    InvalidHeaderBufferLength,
    /// invalid header value or padding
    InvalidHeadOrTail,
    /// unsupported version of the packet
    UnsupportedVersion,
    /// invalid message type
    InvalidMessageType,
    /// invalid topic length
    InvalidTopicLength,
    /// invalid message length
    InvalidMessageLength,
    /// invalid request/response type
    InvalidResuestResponseType,
}

/// structure containing the complete information about a message.
#[derive(Debug, Clone)]
pub struct Msg {
    /// `Header`: the header of the message.
    pub header: Header,
    /// The topic for the message.
    pub topic: String,
    /// the actual message, bytes.
    pub message: Vec<u8>,
    /// `tokio::broadcast::sync::Sender` the channel for passing the messages across.
    pub channel: Option<Sender<Msg>>,
    /// client_id: to identify each socket connection/client.
    pub client_id: Option<String>,
}

impl Msg {
    /// Creates a new `Msg` with the given data.
    pub fn new(pkt_type: PktType, topic: String, message: Option<Vec<u8>>) -> Msg {
        let msg: Vec<u8> = match message {
            Some(m) => m,
            None => vec![],
        };

        Msg {
            header: Header::new(pkt_type, topic.len() as u8, msg.len() as u16),
            topic,
            message: msg,
            channel: None,
            client_id: None,
        }
    }

    /// adds the given channel to the message.
    pub fn channel(&mut self, chan: Sender<Msg>) {
        self.channel = Some(chan);
    }

    // returns the client id for the message.
    pub fn client_id(&mut self, client_id: String) {
        self.client_id = Some(client_id);
    }

    /// generates the response `Msg` with the given data.
    pub fn response_msg(&self, _message: Vec<u8>) -> Result<Msg, String> {
        let mut header: Header = match self.header.response_header() {
            Ok(h) => h,
            Err(e) => {
                error!("unable to generate the response header: {:?}", e);
                return Err("unable to generate response header".to_string());
            }
        };
        header.message_length = _message.len() as u16;
        Ok(Msg {
            header,
            topic: self.topic.clone(),
            message: _message,
            channel: None,
            client_id: None,
        })
    }

    /// returns bytes for the `Msg` that can be sent to the stream.
    pub fn bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = self.header.bytes().to_vec();
        buffer.extend(self.topic.as_bytes().to_vec());
        buffer.extend(self.message.clone());
        trace!("the generated buffer is: {:?}", buffer);
        buffer
    }
}

/// returns a response `Msg`.
pub fn get_msg_response(msg: Msg) -> Result<Vec<u8>, String> {
    let mut resp: Vec<u8> = match msg.response_msg(msg.message.clone()) {
        Ok(m) => m.header.bytes().to_vec(),
        Err(e) => {
            error!(
                "error occurred while generating the response message: {}",
                e
            );
            return Err(e);
        }
    };
    resp.extend(msg.topic.bytes());
    Ok(resp)
}
