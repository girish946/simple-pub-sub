use crate::message;
use crate::stream;
use crate::PktType;
use anyhow::Result;
use log::{error, info, warn};
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast::Sender;
use uuid;

/// reads a `Msg` from the channel.
pub async fn read_channel_msg(
    chan: Sender<message::Msg>,
) -> Result<message::Msg, tokio::sync::broadcast::error::RecvError> {
    let mut rx = chan.subscribe();
    rx.recv().await
}

/// Handles the communication between a client and the broker.
pub async fn handle_client<S>(mut socket: S, chan: Sender<message::Msg>)
where
    S: AsyncWriteExt + Unpin + Send + tokio::io::AsyncReadExt + 'static,
{
    let client_chan = tokio::sync::broadcast::channel(1).0;
    let client_id = uuid::Uuid::new_v4().to_string();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = stream::read_message(&mut socket) => {
                    match msg {
                        Ok(mut m) => {
                            m.client_id(client_id.clone());
                            if !m.topic.is_empty() {
                                info!("topic: {}", m.topic);
                                match m.header.pkt_type {
                                    PktType::PUBLISH | PktType::SUBSCRIBE | PktType::UNSUBSCRIBE | PktType::QUERY => {
                                        m.channel(client_chan.clone());
                                        if let Err(e) = chan.send(m.clone()) {
                                            error!("error while sending message: {:?}", e);
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            if m.header.pkt_type != PktType::QUERY {
                                if let Ok(v) = message::get_msg_response(m.clone()) {
                                    if let Err(e) = socket.write_all(&v).await {
                                        error!("could not write the data to the socket: {:?}", e);
                                    }
                                } else {
                                    error!("error while writing the data to the socket");
                                }
                            }
                        },
                        Err(_e) => {
                            warn!("client disconnected: {}", client_id);
                            return;
                        }
                    }
                },
                chan_msg = read_channel_msg(client_chan.clone()) => {
                    if let Ok(m) = chan_msg {
                        info!("message received: {:?}, {}", m.topic.clone(), m.message.len());
                        if let Err(e) = socket.write_all(&m.bytes()).await {
                            error!("failed to write data to socket: {:?}", e);
                        }
                    }
                }
            }
        }
    });
}
