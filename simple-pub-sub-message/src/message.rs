use crate::{header::Header, PktType};
use anyhow::{bail, Result};
use log::trace;
use tokio::sync::broadcast::Sender;

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
    /// ```
    /// use simple_pub_sub_message::message::Msg;
    /// use simple_pub_sub_message::PktType;
    /// let msg = Msg::new(PktType::PUBLISH, "test".to_string(), Some(b"the message".to_vec()));
    /// ```
    pub fn new(pkt_type: PktType, topic: String, message: Option<Vec<u8>>) -> Msg {
        let msg: Vec<u8> = message.unwrap_or_default();

        Msg {
            header: Header::new(pkt_type, topic.len() as u8, msg.len() as u16),
            topic,
            message: msg,
            channel: None,
            client_id: None,
        }
    }

    /// adds the given channel to the message.
    /// ```
    /// use simple_pub_sub_message::message::Msg;
    /// use simple_pub_sub_message::PktType;
    /// use tokio::sync::broadcast::Sender;
    /// let mut msg = Msg::new(PktType::PUBLISH, "test".to_string(), Some(b"the message".to_vec()));
    /// let chan: tokio::sync::broadcast::Sender<Msg> =
    ///   tokio::sync::broadcast::Sender::new(1);
    /// msg.channel(chan)
    /// ```
    pub fn channel(&mut self, chan: Sender<Msg>) {
        self.channel = Some(chan);
    }

    ///returns the client id for the message.
    /// ```
    /// use simple_pub_sub_message::message::Msg;
    /// use simple_pub_sub_message::PktType;
    /// use uuid;
    /// let mut msg = Msg::new(PktType::PUBLISH, "test".to_string(), Some(b"the message".to_vec()));
    /// let client_id = uuid::Uuid::new_v4().to_string();
    /// msg.client_id(client_id)
    /// ```
    pub fn client_id(&mut self, client_id: String) {
        self.client_id = Some(client_id);
    }

    /// generates the response `Msg` with the given data.
    /// ```
    /// use simple_pub_sub_message::message::Msg;
    /// use simple_pub_sub_message::PktType;
    /// let mut msg = Msg::new(PktType::PUBLISH, "test".to_string(), Some(b"the message".to_vec()));
    /// let response_msg = msg.response_msg(vec![]);
    /// ```
    pub fn response_msg(&self, message: Vec<u8>) -> Result<Msg> {
        let mut header: Header = self.header.response_header()?;
        header.message_length = message.len() as u16;
        Ok(Msg {
            header,
            topic: self.topic.clone(),
            message,
            channel: None,
            client_id: None,
        })
    }

    /// returns bytes for the `Msg` that can be sent to the stream.
    ///```
    /// use simple_pub_sub_message::message::Msg;
    /// use simple_pub_sub_message::PktType;
    /// let mut msg = Msg::new(PktType::PUBLISH, "test".to_string(), Some(b"the message".to_vec()));
    /// let bytes = msg.bytes();
    /// ```
    pub fn bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = self.header.bytes().to_vec();
        buffer.extend(self.topic.as_bytes().to_vec());
        buffer.extend(self.message.clone());
        trace!("the generated buffer is: {:?}", buffer);
        buffer
    }
}

/// returns a response `Msg`.
/// ```
/// use simple_pub_sub_message::message::Msg;
/// use simple_pub_sub_message::PktType;
/// use simple_pub_sub_message::message::get_msg_response;
/// let mut msg = Msg::new(PktType::PUBLISHACK, "test".to_string(), Some(b"".to_vec()));
/// let response_msg = get_msg_response(msg);
/// ```

pub fn get_msg_response(msg: Msg) -> Result<Vec<u8>> {
    let mut resp: Vec<u8> = msg.response_msg(msg.message.clone())?.bytes();
    resp.extend(msg.topic.bytes());
    Ok(resp)
}

impl PartialEq for Msg {
    fn eq(&self, other: &Self) -> bool {
        if self.header == other.header && self.topic == other.topic && self.message == other.message
        {
            return true;
        }
        false
    }
}
impl TryFrom<&[u8]> for Msg {
    type Error = anyhow::Error;

    /// Parses a `Msg` from a `Vec<u8>`.
    /// ```
    /// use simple_pub_sub_message::message::Msg;
    /// let buf = [15, 0, 1, 2, 3, 0, 12, 0, 97, 98,
    ///   99, 116, 101, 115, 116, 32, 109, 101, 115,
    ///   115, 97, 103, 101];
    /// let msg = Msg::try_from(buf.as_ref()).unwrap();
    /// println!("{:?}", msg);
    /// ```

    fn try_from(bytes: &[u8]) -> Result<Msg> {
        let header = Header::try_from(bytes[..8].as_ref())?;
        let topic: String = String::from_utf8(bytes[8..(8 + header.topic_length).into()].to_vec())?;
        let message_end: usize = ((8 + header.topic_length) as u16 + header.message_length).into();

        if bytes.len() < message_end {
            bail!("invalid Msg length");
        }
        let message = bytes[(8 + header.topic_length).into()..message_end].to_vec();
        Ok(Msg {
            header,
            topic,
            message,
            channel: None,
            client_id: None,
        })
    }
}
impl TryFrom<Vec<u8>> for Msg {
    type Error = anyhow::Error;

    /// Parses a `Msg` from a `Vec<u8>`.
    /// ```
    /// use simple_pub_sub_message::message::Msg;
    /// let buf = vec![15, 0, 1, 2, 3, 0, 12, 0, 97, 98,
    ///   99, 116, 101, 115, 116, 32, 109, 101, 115,
    ///   115, 97, 103, 101];
    /// let msg = Msg::try_from(buf).unwrap();
    /// ```
    fn try_from(bytes: Vec<u8>) -> Result<Msg> {
        Msg::try_from(bytes.as_ref())
    }
}
