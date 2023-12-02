use crate::message::{self, Msg};
use log::{error, info};
use std::collections::HashMap;
use std::vec;
use tokio;
use tokio::sync::broadcast::Sender;

#[derive(Debug, Clone)]
pub struct TopicMap {
    pub map: HashMap<String, Vec<Sender<Msg>>>,
}
impl TopicMap {
    pub fn add_channel(&mut self, topic: String, channel: Sender<Msg>) {
        if self.map.contains_key(&topic) {
            match self.map.get_mut(&topic) {
                Some(channels) => channels.push(channel),
                None => {}
            }
        } else {
            self.map.insert(topic, vec![channel]);
        }
    }
    pub fn remove_channel(&mut self, topic: String, channel: Sender<Msg>) {
        if self.map.contains_key(&topic) {
            match self.map.get_mut(&topic) {
                Some(channels) => {
                    for (pos, e) in channels.to_owned().iter().enumerate() {
                        if e.same_channel(&channel) {
                            channels.remove(pos);
                            break;
                        }
                    }
                }
                None => {}
            };
            info!("channels: {:?}", self.map);
        }
    }
    pub fn add_topic(&mut self, topic: String) {
        if !self.map.contains_key(&topic) {
            let v: Vec<Sender<Msg>> = Vec::new();
            self.map.insert(topic, v);
        }
    }

    pub async fn publish(&mut self, msg: Msg) {
        if !self.map.contains_key(&msg.topic) {
            return;
        }
        let mut dead_channels: Vec<Sender<Msg>> = Vec::new();

        match self.map.get_mut(&msg.topic) {
            Some(channels) => {
                for ch in channels {
                    info!("publishing to :{:?}", ch);
                    match ch.send(msg.clone()) {
                        Ok(n) => n,
                        Err(e) => {
                            error!(
                                "could not publish to topic: {}: {}",
                                msg.topic,
                                e.to_string()
                            );
                            dead_channels.push(ch.clone());
                            0
                        }
                    };
                }
            }
            None => {}
        };
        for ch in dead_channels {
            self.remove_channel(msg.topic.clone(), ch);
        }
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
                            map.add_channel(msg.topic, msg.channel.unwrap());
                            info!("map: {:?}", map);
                        }
                        message::PktType::UNSUBSCRIBE => {
                            info!("unsubscribing:");
                            map.remove_channel(msg.topic, msg.channel.unwrap());
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
