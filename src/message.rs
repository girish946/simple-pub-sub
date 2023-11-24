use log::error;
const REGISTER: u8 = 0x01;
const PUBLISH: u8 = 0x02;
const SUBSCRIBE: u8 = 0x03;
const UNSUBSCRIBE: u8 = 0x04;
const QUERY: u8 = 0x05;

#[derive(Debug, Clone)]
pub enum PktType {
    REGISTER,
    PUBLISH,
    SUBSCRIBE,
    UNSUBSCRIBE,
    QUERY,
}
impl ToString for PktType {
    fn to_string(&self) -> String {
        match self {
            PktType::REGISTER => "REGISTER".to_string(),
            PktType::PUBLISH => "PUBLISH".to_string(),
            PktType::SUBSCRIBE => "SUBSCRIBE".to_string(),
            PktType::UNSUBSCRIBE => "UNSUBSCRIBE".to_string(),
            PktType::QUERY => "QUERY".to_string(),
        }
    }
}
pub const SUPPORTED_VERSIONS: [[u8; 2]; 1] = [[0x00, 0x01]];

#[derive(Debug)]
pub enum HeaderError {
    InvalidHeaderBufferLength,
    InvalidHeadOrTail,
    UnsupportedVersion,
    InvalidMessageType,
    InvalidTopicLength,
    InvalidMessageLength,
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

impl Header {
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
        return Ok(Header {
            header: 0x0F,
            version: [bytes[1], bytes[2]],
            pkt_type,
            topic_length: bytes[4],
            message_length,
            padding: 0x00,
        });
    }
}
