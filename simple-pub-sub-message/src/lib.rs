pub mod error;
pub mod header;
pub mod message;
pub mod pkt;
pub use pkt::PktType;

pub mod constants {

    /// supported versions for the pub-sub header format/protocol.
    pub const SUPPORTED_VERSIONS: [[u8; 2]; 1] = [[0x00, 0x01]];

    /// default version for the pub-sub header format/protocol.
    pub const DEFAULT_VERSION: [u8; 2] = [0x00, 0x01];

    /// the header length
    pub const HEADER_LEN: usize = 8;

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
}

#[cfg(test)]
mod tests {
    use log::info;

    use crate::header::Header;

    // use super::*;

    #[test]
    fn header_parse_pass() {
        // The test header is
        // Header { header: 15, version: [0, 1], pkt_type: PUBLISH, topic_length: 3, message_length: 12, padding: 0 }
        assert!(Header::try_from(vec![
            15, // `HEADER_BYTE`
            0, 1, // `VERSION_BYTE_0`, `VERSION_BYTE_1`
            2, // `PktType`
            3, // `TOPIC_LENGTH_BYTE`
            0, 12, // `MESSAGE_LENGTH_BYTE_0`, `MESSAGE_LENGTH_BYTE_1`
            0,  // `PADDING_BYTE`
        ])
        .is_ok());
    }

    #[test]
    fn header_parse_fail() {
        // The test header is
        // Header { header: 16, version: [0, 1], pkt_type: PUBLISH, topic_length: 3, message_length: 12, padding: 0 }
        assert!(Header::try_from(vec![
            16, // `HEADER_BYTE`
            0, 1, // `VERSION_BYTE_0`, `VERSION_BYTE_1`
            2, // `PktType`
            3, // `TOPIC_LENGTH_BYTE`
            0, 12, // `MESSAGE_LENGTH_BYTE_0`, `MESSAGE_LENGTH_BYTE_1`
            0,  // `PADDING_BYTE`
        ])
        .is_err());
    }

    #[test]
    fn message_parse_pass() {
        use crate::message::Msg;

        let buf = [
            15, 0, 1, 2, 3, 0, 12, 0, 97, 98, 99, 116, 101, 115, 116, 32, 109, 101, 115, 115, 97,
            103, 101,
        ];
        let msg = Msg::try_from(buf.as_ref());
        info!("{:?}", msg);
        assert!(msg.is_ok());
        info!("{:?}", msg.unwrap());
    }

    #[test]
    fn message_parse_fail() {
        use crate::message::Msg;
        env_logger::init();
        let buf = [
            15, 0, 1, 2, 3, 0, 12, 0, 97, 98, 99, 116, 101, 115, 116, 32, 109, 101, 115, 115, 97,
            103,
        ];
        let msg = Msg::try_from(buf.as_ref());
        assert!(msg.is_err());
    }
}
