use crate::{header::Header, PktType};
use anyhow::Result;
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
    pub fn response_msg(&self, _message: Vec<u8>) -> Result<Msg> {
        let mut header: Header = self.header.response_header()?;
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
pub fn get_msg_response(msg: Msg) -> Result<Vec<u8>> {
    let mut resp: Vec<u8> = msg.response_msg(msg.message.clone())?.bytes();
    resp.extend(msg.topic.bytes());
    Ok(resp)
}
