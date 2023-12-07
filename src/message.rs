use log::error;
use tokio::sync::broadcast::Sender;
pub const REGISTER: u8 = 0x01;
pub const PUBLISH: u8 = 0x02;
pub const SUBSCRIBE: u8 = 0x03;
pub const UNSUBSCRIBE: u8 = 0x04;
pub const QUERY: u8 = 0x05;
pub const REGISTERACK: u8 = 0x0A;
pub const PUBLISHACK: u8 = 0x0B;
pub const SUBSCRIBEACK: u8 = 0x0C;
pub const UNSUBSCRIBEACK: u8 = 0x0D;
pub const QUERYRESP: u8 = 0x0E;

#[derive(Debug, Clone)]
pub enum PktType {
    REGISTER,
    PUBLISH,
    SUBSCRIBE,
    UNSUBSCRIBE,
    QUERY,
    REGISTERACK,
    PUBLISHACK,
    SUBSCRIBEACK,
    UNSUBSCRIBEACK,
    QUERYRESP,
}
impl PktType {
    pub fn to_byte(&self) -> u8 {
        match self {
            PktType::REGISTER => REGISTER,
            PktType::PUBLISH => PUBLISH,
            PktType::SUBSCRIBE => SUBSCRIBE,
            PktType::UNSUBSCRIBE => UNSUBSCRIBE,
            PktType::QUERY => QUERY,
            PktType::REGISTERACK => REGISTERACK,
            PktType::PUBLISHACK => PUBLISHACK,
            PktType::SUBSCRIBEACK => SUBSCRIBEACK,
            PktType::UNSUBSCRIBEACK => UNSUBSCRIBEACK,
            PktType::QUERYRESP => QUERYRESP,
        }
    }
}
impl ToString for PktType {
    fn to_string(&self) -> String {
        match self {
            PktType::REGISTER => "REGISTER".to_string(),
            PktType::PUBLISH => "PUBLISH".to_string(),
            PktType::SUBSCRIBE => "SUBSCRIBE".to_string(),
            PktType::UNSUBSCRIBE => "UNSUBSCRIBE".to_string(),
            PktType::QUERY => "QUERY".to_string(),
            PktType::REGISTERACK => "REGISTER_ACK".to_string(),
            PktType::PUBLISHACK => "PUBLISH_ACK".to_string(),
            PktType::SUBSCRIBEACK => "SUBSCRIBE_ACK".to_string(),
            PktType::UNSUBSCRIBEACK => "UNSUBSCRIBE_ACK".to_string(),
            PktType::QUERYRESP => "QUERY_RESP".to_string(),
        }
    }
}

pub const SUPPORTED_VERSIONS: [[u8; 2]; 1] = [[0x00, 0x01]];

#[derive(Debug, Clone)]
pub enum HeaderError {
    InvalidHeaderBufferLength,
    InvalidHeadOrTail,
    UnsupportedVersion,
    InvalidMessageType,
    InvalidTopicLength,
    InvalidMessageLength,
    InvalidResuestResponseType,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub header: u8,
    pub version: [u8; 2],
    pub pkt_type: PktType,
    pub topic_length: u8,
    pub message_length: u16,
    pub padding: u8,
}

#[derive(Debug, Clone)]
pub struct Msg {
    pub header: Header,
    pub topic: String,
    pub message: Vec<u8>,
    pub channel: Option<Sender<Msg>>,
    pub client_id: Option<String>,
}

impl Msg {
    pub fn channel(&mut self, chan: Sender<Msg>) {
        self.channel = Some(chan);
    }

    pub fn response_msg(&self, _message: Vec<u8>) -> Result<Msg, String> {
        let header: Header = match self.header.response_header() {
            Ok(h) => h,
            Err(e) => {
                error!("unable to generate the response header: {:?}", e);
                return Err("unable to genreate response header".to_string());
            }
        };
        return Ok(Msg {
            header,
            topic: self.topic.clone(),
            message: "".as_bytes().to_vec(),
            channel: None,
            client_id: None,
        });
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = self.header.bytes();
        buffer.extend(self.topic.as_bytes().to_vec());
        buffer.extend(self.message.clone());
        // info!("the generated buffer is: {:?}", buffer);
        buffer
    }
}

impl Header {
    pub fn response_header(&self) -> Result<Header, HeaderError> {
        let resp_type: PktType = match self.pkt_type {
            PktType::SUBSCRIBE => PktType::SUBSCRIBEACK,
            PktType::PUBLISH => PktType::PUBLISHACK,
            PktType::UNSUBSCRIBE => PktType::UNSUBSCRIBEACK,
            PktType::REGISTER => PktType::REGISTERACK,
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
            REGISTER => PktType::REGISTER,
            PUBLISH => PktType::PUBLISH,
            SUBSCRIBE => PktType::SUBSCRIBE,
            UNSUBSCRIBE => PktType::UNSUBSCRIBE,
            QUERY => PktType::QUERY,
            _ => {
                error!("invalid message type, aborting");
                return Err(HeaderError::InvalidMessageType);
            }
        };

        if bytes[4] == 0 {
            match pkt_type {
                PktType::REGISTER => {
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

pub fn get_msg_response(msg: Msg) -> Result<Vec<u8>, String> {
    let mut resp: Vec<u8> = match msg.response_msg(msg.message.clone()) {
        Ok(m) => m.header.bytes(),
        Err(e) => {
            error!("error occured while generating the response message: {}", e);
            return Err(e);
        }
    };
    resp.extend(msg.topic.bytes());
    Ok(resp)
}
