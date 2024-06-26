use log::{error, trace};
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

/// Packet type
#[derive(Debug, Clone, PartialEq)]
pub enum PktType {
    /// publish
    PUBLISH,
    /// subscribe
    SUBSCRIBE,
    /// unsubscribe
    UNSUBSCRIBE,
    /// query the topics
    QUERY,
    /// acknoledgement to publish
    PUBLISHACK,
    /// acknoledgement to subscribe
    SUBSCRIBEACK,
    /// acknoledgement to unsubscribe
    UNSUBSCRIBEACK,
    /// response to the query packet
    QUERYRESP,
}
impl PktType {
    /// returns the byte value for a given packet type.
    pub fn to_byte(&self) -> u8 {
        match self {
            PktType::PUBLISH => PUBLISH,
            PktType::SUBSCRIBE => SUBSCRIBE,
            PktType::UNSUBSCRIBE => UNSUBSCRIBE,
            PktType::QUERY => QUERY,
            PktType::PUBLISHACK => PUBLISHACK,
            PktType::SUBSCRIBEACK => SUBSCRIBEACK,
            PktType::UNSUBSCRIBEACK => UNSUBSCRIBEACK,
            PktType::QUERYRESP => QUERYRESP,
        }
    }
}
impl ToString for PktType {
    /// returns the string representation for the given packet type.
    fn to_string(&self) -> String {
        match self {
            PktType::PUBLISH => "PUBLISH".to_string(),
            PktType::SUBSCRIBE => "SUBSCRIBE".to_string(),
            PktType::UNSUBSCRIBE => "UNSUBSCRIBE".to_string(),
            PktType::QUERY => "QUERY".to_string(),
            PktType::PUBLISHACK => "PUBLISH_ACK".to_string(),
            PktType::SUBSCRIBEACK => "SUBSCRIBE_ACK".to_string(),
            PktType::UNSUBSCRIBEACK => "UNSUBSCRIBE_ACK".to_string(),
            PktType::QUERYRESP => "QUERY_RESP".to_string(),
        }
    }
}

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

/// Header for the pub/sub packet
/// total length 8 bytes.
#[derive(Debug, Clone)]
pub struct Header {
    /// start byte of the packet, default value: 0x0F
    pub header: u8,
    /// pub-sub version: two bytes.
    pub version: [u8; 2],
    /// packet type: `PktType`
    pub pkt_type: PktType,
    /// topic length for publishing/subscribing/querying.
    pub topic_length: u8,
    /// message length: Max length 16 MB.
    pub message_length: u16,
    /// padding/endo of the header: 0x00
    pub padding: u8,
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
        let mut buffer: Vec<u8> = self.header.bytes();
        buffer.extend(self.topic.as_bytes().to_vec());
        buffer.extend(self.message.clone());
        trace!("the generated buffer is: {:?}", buffer);
        buffer
    }
}

impl Header {
    /// creates a new `Header` with the given data.
    pub fn new(pkt_type: PktType, topic_len: u8, message_len: u16) -> Header {
        Header {
            header: 0x0F,
            version: DEFAULT_VERSION,
            pkt_type,
            topic_length: topic_len,
            message_length: message_len,
            padding: 0x00,
        }
    }

    /// returns a `Header` for the response `Msg`.
    pub fn response_header(&self) -> Result<Header, HeaderError> {
        let resp_type: PktType = match self.pkt_type {
            PktType::SUBSCRIBE => PktType::SUBSCRIBEACK,
            PktType::PUBLISH => PktType::PUBLISHACK,
            PktType::UNSUBSCRIBE => PktType::UNSUBSCRIBEACK,
            PktType::QUERY => PktType::QUERYRESP,
            _ => {
                error!("invalid request/response type");
                return Err(HeaderError::InvalidResuestResponseType);
            }
        };
        Ok(Header {
            header: 0x0F,
            version: self.version,
            pkt_type: resp_type,
            topic_length: self.topic_length,
            message_length: self.message_length,
            padding: 0x00,
        })
    }

    /// returns the bytes for `Header`.
    pub fn bytes(&self) -> Vec<u8> {
        let bytes_ = self.message_length.to_be_bytes();
        vec![
            self.header,
            self.version[0],
            self.version[1],
            self.pkt_type.to_byte(),
            self.topic_length,
            bytes_[0],
            bytes_[1],
            self.padding,
        ]
    }

    /// returns a `Header` from bytes.
    pub fn from_vec(bytes: Vec<u8>) -> Result<Header, HeaderError> {
        if !bytes.len() == 8 {
            error!("invalid header buffer length, aborting");
            return Err(HeaderError::InvalidHeaderBufferLength);
        }

        if !(bytes[0] == 0x0F && bytes[7] == 0x00) {
            error!("invalid header value, aborting");
            return Err(HeaderError::InvalidHeadOrTail);
        }

        if !SUPPORTED_VERSIONS.contains(&[bytes[1], bytes[2]]) {
            error!("unsupported packet version, aborting");
            return Err(HeaderError::UnsupportedVersion);
        }

        let pkt_type: PktType = match bytes[3] {
            PUBLISH => PktType::PUBLISH,
            SUBSCRIBE => PktType::SUBSCRIBE,
            UNSUBSCRIBE => PktType::UNSUBSCRIBE,
            QUERY => PktType::QUERY,
            PUBLISHACK => PktType::PUBLISHACK,
            SUBSCRIBEACK => PktType::SUBSCRIBEACK,
            QUERYRESP => PktType::QUERYRESP,
            _ => {
                error!("invalid message type, aborting");
                return Err(HeaderError::InvalidMessageType);
            }
        };

        if bytes[4] == 0 {
            match pkt_type {
                PktType::PUBLISH => {
                    error!("invalid topic length, aborting");
                    return Err(HeaderError::InvalidTopicLength);
                }
                PktType::SUBSCRIBE => {
                    error!("invalid topic length, aborting");
                    return Err(HeaderError::InvalidTopicLength);
                }
                PktType::UNSUBSCRIBE => {
                    error!("invalid topic length, aborting");
                    return Err(HeaderError::InvalidTopicLength);
                }
                PktType::QUERY => {
                    error!("invalid topic length, aborting");
                    return Err(HeaderError::InvalidTopicLength);
                }
                _ => {}
            };
        }

        let message_length = ((bytes[5] as u16) << 8) | bytes[6] as u16;

        if message_length == 0 {
            match pkt_type {
                PktType::PUBLISH => {
                    error!("invalid message length, aborting");
                    return Err(HeaderError::InvalidMessageLength);
                }
                PktType::QUERY => {
                    error!("invalid message length, aborting");
                    return Err(HeaderError::InvalidMessageLength);
                }
                _ => {}
            };
        }
        Ok(Header {
            header: 0x0F,
            version: [bytes[1], bytes[2]],
            pkt_type,
            topic_length: bytes[4],
            message_length,
            padding: 0x00,
        })
    }
}

/// returns a response `Msg`.
pub fn get_msg_response(msg: Msg) -> Result<Vec<u8>, String> {
    let mut resp: Vec<u8> = match msg.response_msg(msg.message.clone()) {
        Ok(m) => m.header.bytes(),
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
