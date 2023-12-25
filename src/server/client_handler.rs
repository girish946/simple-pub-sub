use crate::message;
use crate::stream;
use log::{error, info, warn};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast::Sender;
use uuid;

/// reads a `Msg` from the channel.
pub async fn read_channel_msg(
    chan: Sender<message::Msg>,
) -> Result<message::Msg, tokio::sync::broadcast::error::RecvError> {
    let mut rx = chan.subscribe();
    rx.recv().await
}

/// Handels the communication between a client and the broker.
pub async fn handle_clinet(mut socket: TcpStream, chan: Sender<message::Msg>) {
    let client_chan: tokio::sync::broadcast::Sender<message::Msg> =
        tokio::sync::broadcast::Sender::new(1);
    let client_id = uuid::Uuid::new_v4().to_string();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                msg_ = stream::read_message(&mut socket) =>{
                    match msg_{
                        Ok(mut m)=>{
                            m.client_id(client_id.clone());
                            if !m.topic.is_empty() {
                                info!("topic: {}", m.topic);
                                match m.header.pkt_type{
                                    message::PktType::PUBLISH=>{
                                        info!("it's a publish packet");
                                        match chan.send(m.clone()) {
                                            Ok(n) => n,
                                            Err(e) => {
                                                error!("error while checking the topic in the map: {}",e.to_string());
                                                0
                                            }
                                        };
                                    },
                                    message::PktType::SUBSCRIBE=>{
                                        info!("it's a subscribe pkt attaching the channel");
                                        m.channel(client_chan.clone());
                                        match chan.send(m.clone()) {
                                            Ok(n) => n,
                                            Err(e) => {
                                                error!("error while checking the topic in the map: {}",e.to_string());
                                                0
                                            }
                                        };
                                    }
                                    message::PktType::UNSUBSCRIBE=> {
                                        m.channel(client_chan.clone());
                                        match chan.send(m.clone()) {
                                            Ok(n) => n,
                                            Err(e) => {
                                                error!("error while checking the topic in the map: {}", e.to_string());
                                                0
                                            },
                                        };
                                    },
                                    message::PktType::QUERY=>{
                                        m.channel(client_chan.clone());
                                        match chan.send(m.clone()) {
                                            Ok(n) => n,
                                            Err(e) => {
                                                error!("error while checking the topic in the map: {}", e.to_string());
                                                0
                                            },
                                        };
                                    },
                                    _=>{}
                                };
                            }
                           if  m.header.pkt_type != message::PktType::QUERY{
                               match message::get_msg_response(m.clone()){
                                   Ok(v)=>{
                                       match socket.write_all(&v).await{
                                           Ok(_)=>{},
                                           Err(e)=>{
                                               error!("could not write the data to the socket: {}", e.to_string());
                                           }
                                       }
                                   },
                                   Err(e)=>{
                                       error!("error while writing the data to the socket: {}", e.to_string());
                                   }
                               }
                           }
                        },
                        Err(_e)=>{
                            warn!("client disconnected: {}", client_id);
                            return;
                        }
                    };
                },
                chan_msg = read_channel_msg(client_chan.clone())=>{
                    match chan_msg {
                        Ok(m) => {
                            info!("message received: {:?}, {}", m.topic.clone(), m.message.len());
                            socket
                                    .write_all(&m.bytes())
                                    .await
                                    .expect("failed to write data to socket");
                        },
                        Err(_e) => {},
                    }
                }
            }
        }
    });
}
