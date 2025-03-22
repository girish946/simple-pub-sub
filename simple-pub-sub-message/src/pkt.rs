use crate::constants::*;
use std::fmt::Display;

/// Packet type
#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum PktType {
    /// publish
    PUBLISH = PUBLISH,
    /// subscribe
    SUBSCRIBE = SUBSCRIBE,
    /// unsubscribe
    UNSUBSCRIBE = UNSUBSCRIBE,
    /// query the topics
    QUERY = QUERY,
    /// acknoledgement to publish
    PUBLISHACK = PUBLISHACK,
    /// acknoledgement to subscribe
    SUBSCRIBEACK = SUBSCRIBEACK,
    /// acknoledgement to unsubscribe
    UNSUBSCRIBEACK = UNSUBSCRIBEACK,
    /// response to the query packet
    QUERYRESP = QUERYRESP,
}

impl PktType {
    /// returns the byte for the given type of packet
    /// ```
    /// use simple_pub_sub_message::pkt::PktType;
    /// let publish_byte = PktType::PUBLISH.byte();
    /// assert_eq!(publish_byte, simple_pub_sub_message::constants::PUBLISH);
    ///
    /// ```
    pub fn byte(&self) -> u8 {
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

impl Display for PktType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pkt = match self {
            PktType::PUBLISH => "PUBLISH".to_string(),
            PktType::SUBSCRIBE => "SUBSCRIBE".to_string(),
            PktType::UNSUBSCRIBE => "UNSUBSCRIBE".to_string(),
            PktType::QUERY => "QUERY".to_string(),
            PktType::PUBLISHACK => "PUBLISH_ACK".to_string(),
            PktType::SUBSCRIBEACK => "SUBSCRIBE_ACK".to_string(),
            PktType::UNSUBSCRIBEACK => "UNSUBSCRIBE_ACK".to_string(),
            PktType::QUERYRESP => "QUERY_RESP".to_string(),
        };
        write!(f, "{}", pkt)
    }
}
