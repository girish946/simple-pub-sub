/// Header of the simple_pub_sub_message packet
use crate::{
    constants::{self, *},
    error::HeaderError,
    PktType,
};
use anyhow::{anyhow, bail, Result};

/// byte at index 0
/// indicate the start of header
const HEADER_START: usize = 0;

/// byte at index 7
/// indicate the end of header
const HEADER_END: usize = 7;

/// version is indicated using two bytes,
/// first byte of the version
/// byte at index 1
const VERSION_BYTE_0: usize = 1;

/// version is indicated using two bytes,
/// second byte of the version
/// byte at index 2
const VERSION_BYTE_1: usize = 2;

/// byte that indicates the packet type
/// byte at index 3
const PACKET_BYTE: usize = 3;

/// byte that indicates the topic length
/// byte at index 4
const TOPIC_LENGTH_BYTE: usize = 4;

/// first byte of the message length (big endian)
/// byte at index 5
const MESSAGE_LENGTH_BYTE_0: usize = 5;

/// second byte of the message length (big endian)
/// byte at index 6
const MESSAGE_LENGTH_BYTE_1: usize = 6;

/// start of the header
/// value: 0x0F
const HEADER_BYTE: u8 = 0x0F;

/// end of header
/// value: 0x00
const PADDING_BYTE: u8 = 0x00;

/// Header for the pub/sub packet
/// total length 8 bytes.
#[derive(Debug, Clone, PartialEq)]
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

impl Header {
    /// creates a new `Header` with the given data.
    /// ```
    /// use simple_pub_sub_message::header::Header;
    /// use simple_pub_sub_message::PktType;
    /// Header::new(PktType::PUBLISH, 8, 20);
    /// ```
    pub fn new(pkt_type: PktType, topic_len: u8, message_len: u16) -> Header {
        Header {
            header: HEADER_BYTE,
            version: DEFAULT_VERSION,
            pkt_type,
            topic_length: topic_len,
            message_length: message_len,
            padding: PADDING_BYTE,
        }
    }

    /// returns a `Header` for the response `Msg`.
    /// ```
    /// use simple_pub_sub_message::header::Header;
    /// use simple_pub_sub_message::PktType;
    /// let header = Header::new(PktType::PUBLISH, 8, 20);
    /// let response_header = header.response_header();
    ///```
    pub fn response_header(&self) -> Result<Header> {
        let resp_type: PktType = match self.pkt_type {
            PktType::SUBSCRIBE => PktType::SUBSCRIBEACK,
            PktType::PUBLISH => PktType::PUBLISHACK,
            PktType::UNSUBSCRIBE => PktType::UNSUBSCRIBEACK,
            PktType::QUERY => PktType::QUERYRESP,
            _ => {
                return Err(anyhow!(HeaderError::InvalidResponseType));
            }
        };
        Ok(Header {
            header: HEADER_BYTE,
            version: self.version,
            pkt_type: resp_type,
            topic_length: self.topic_length,
            message_length: self.message_length,
            padding: PADDING_BYTE,
        })
    }

    /// returns the bytes for `Header`.
    /// ```
    /// use simple_pub_sub_message::header::Header;
    /// use simple_pub_sub_message::PktType;
    /// let header = Header::new(PktType::PUBLISH, 8, 20);
    /// header.bytes();
    ///```

    pub fn bytes(&self) -> [u8; 8] {
        let message_length_bytes = self.message_length.to_be_bytes();
        [
            self.header,
            self.version[0],
            self.version[1],
            self.pkt_type.byte(),
            self.topic_length,
            message_length_bytes[0],
            message_length_bytes[1],
            self.padding,
        ]
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = anyhow::Error;

    /// Parses a `Header` from a `&[u8]`
    /// ```
    /// use simple_pub_sub_message::header::Header;
    /// Header::try_from([
    ///        15,    // HEADER_BYTE`
    ///        0, 1,  // `VERSION_BYTE_0`, `VERSION_BYTE_1`
    ///        2,     // `PktType`
    ///        3,     // `TOPIC_LENGTH_BYTE`
    ///        0, 12, // `MESSAGE_LENGTH_BYTE_0`, `MESSAGE_LENGTH_BYTE_1`
    ///        0,     // `PADDING_BYTE`
    /// ].as_ref());
    /// ```

    fn try_from(bytes: &[u8]) -> Result<Header> {
        if !bytes.len() == constants::HEADER_LEN {
            bail!(HeaderError::InvalidHeaderBufferLength);
        }

        if !(bytes[HEADER_START] == HEADER_BYTE && bytes[HEADER_END] == PADDING_BYTE) {
            bail!(HeaderError::InvalidHeadOrTail);
        }

        if !SUPPORTED_VERSIONS.contains(&[bytes[VERSION_BYTE_0], bytes[VERSION_BYTE_1]]) {
            bail!(HeaderError::UnsupportedVersion);
        }

        let pkt_type: PktType = match bytes[PACKET_BYTE] {
            PUBLISH => PktType::PUBLISH,
            SUBSCRIBE => PktType::SUBSCRIBE,
            UNSUBSCRIBE => PktType::UNSUBSCRIBE,
            QUERY => PktType::QUERY,
            PUBLISHACK => PktType::PUBLISHACK,
            SUBSCRIBEACK => PktType::SUBSCRIBEACK,
            QUERYRESP => PktType::QUERYRESP,
            _ => {
                bail!(HeaderError::InvalidPacketType);
            }
        };

        if bytes[TOPIC_LENGTH_BYTE] == 0 {
            // topic would be absent for the response header
            match pkt_type {
                PktType::PUBLISH | PktType::SUBSCRIBE | PktType::UNSUBSCRIBE | PktType::QUERY => {
                    bail!(HeaderError::InvalidTopicLength);
                }
                _ => {}
            };
        }

        // calculate the message length
        let message_length =
            ((bytes[MESSAGE_LENGTH_BYTE_0] as u16) << 8) | bytes[MESSAGE_LENGTH_BYTE_1] as u16;

        // message length can't be 0 for the publish
        // or the query packet
        if message_length == 0 {
            match pkt_type {
                PktType::PUBLISH => {
                    bail!(HeaderError::InvalidMessageLength(0));
                }
                PktType::QUERY => {
                    bail!(HeaderError::InvalidMessageLength(0));
                }
                _ => {}
            };
        }

        Ok(Header {
            header: HEADER_BYTE,
            version: [bytes[VERSION_BYTE_0], bytes[VERSION_BYTE_1]],
            pkt_type,
            topic_length: bytes[TOPIC_LENGTH_BYTE],
            message_length,
            padding: PADDING_BYTE,
        })
    }
}

impl TryFrom<Vec<u8>> for Header {
    type Error = anyhow::Error;

    /// Parses a `Header` from a `Vec<u8>`.
    /// ```
    /// use simple_pub_sub_message::header::Header;
    /// Header::try_from(vec![
    ///        15,    // `HEADER_BYTE`
    ///        0, 1,  // `VERSION_BYTE_0`, `VERSION_BYTE_1`
    ///        2,     // `PktType`
    ///        3,     // `TOPIC_LENGTH_BYTE`
    ///        0, 12, // `MESSAGE_LENGTH_BYTE_0`, `MESSAGE_LENGTH_BYTE_1`
    ///        0,     // `PADDING_BYTE`
    /// ]);
    /// ```
    fn try_from(bytes: Vec<u8>) -> Result<Header> {
        Header::try_from(&bytes[..])
    }
}
