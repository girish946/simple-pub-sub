use crate::message::{self, Msg};
use log::{error, info};
use std::collections::HashMap;
use tokio;
use tokio::sync::broadcast::Sender;

type ClientChannelMap = HashMap<String, Sender<Msg>>;

#[derive(Debug, Clone)]
pub struct TopicMap {
    pub map: HashMap<String, ClientChannelMap>,
}
impl TopicMap {
    pub fn add_channel(&mut self, topic: String, client_id: String, channel: Sender<Msg>) {
        if self.map.contains_key(&topic) {
            match self.map.get_mut(&topic) {
                Some(channels) => {
                    if !channels.contains_key(&client_id) {
                        channels.insert(client_id, channel);
                    }
                    // Not sure if the channel should be replaced if the key is already present.
                }
                None => {}
            }
        } else {
            let mut client_map = ClientChannelMap::new();
            client_map.insert(client_id, channel);
            self.map.insert(topic, client_map);
        }
    }
    pub fn remove_channel(&mut self, topic: String, client_id: String) {
        if self.map.contains_key(&topic) {
            match self.map.get_mut(&topic) {
                Some(channels) => {
                    channels.remove(&client_id);
                }
                None => {}
            };
            info!("channels: {:?}", self.map);
        }
    }
    pub fn add_topic(&mut self, topic: String) {
        if !self.map.contains_key(&topic) {
            let v = ClientChannelMap::new();
            self.map.insert(topic, v);
        }
    }

    pub async fn publish(&mut self, msg: Msg) {
        if !self.map.contains_key(&msg.topic) {
            return;
        }
        let topic = msg.topic.clone();

        match self.map.get_mut(&topic.clone()) {
            Some(channels) => {
                let dead_channels = channels
                    .iter()
                    .map(|(client_id, channel)| {
                        info!("sending msg to the {}", client_id);
                        match channel.send(msg.clone()) {
                            Ok(_n) => "".to_string(),
                            Err(e) => {
                                error!(
                                    "error occured: {} while sending the message to the channel {}",
                                    e.to_string(),
                                    client_id
                                );
                                error!("cleaing up");
                                client_id.clone()
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                info!("dead_channels: {:?}", dead_channels);
                let _ = dead_channels
                    .iter()
                    .map(|client_id| {
                        self.remove_channel(topic.clone(), client_id.clone());
                    })
                    .collect::<Vec<_>>();
            }
            None => {}
        };
    }
}

pub fn get_global_broadcaster() -> tokio::sync::broadcast::Sender<Msg> {
    info!("creating broadcast channel");
    let (glob_tx, _) = tokio::sync::broadcast::channel(1024);
    glob_tx
}

pub async fn topic_manager(chan: Sender<Msg>) {
    let mut map: TopicMap = TopicMap {
        map: HashMap::new(),
    };
    let mut rx = chan.subscribe();
    loop {
        match rx.recv().await {
            Ok(msg) => {
                if !msg.topic.is_empty() {
                    info!("topic received: {}", msg.topic);
                    match msg.header.pkt_type {
                        message::PktType::PUBLISH => {
                            info!("publishing to map:{:?}", map);
                            map.publish(msg).await;
                        }
                        message::PktType::SUBSCRIBE => {
                            map.add_channel(
                                msg.topic,
                                msg.client_id.unwrap(),
                                msg.channel.unwrap(),
                            );
                            info!("map: {:?}", map);
                        }
                        message::PktType::UNSUBSCRIBE => {
                            info!("unsubscribing:");
                            map.remove_channel(msg.topic, msg.client_id.unwrap());
                        }
                        _ => {}
                    };
                }
            }
            Err(e) => {
                error!("error occured while receving the topic: {}", e.to_string());
                // "".to_string()
            }
        };
    }
}
