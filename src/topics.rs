use crate::message::{self, Msg};
use log::{error, info, trace};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use tokio;
use tokio::sync::broadcast::Sender;

type ClientChannelMap = HashMap<String, Sender<Msg>>;

/// The `TopicMap` struct is used to store the channels for a given topic.
#[derive(Debug, Clone)]
pub struct TopicMap {
    pub map: BTreeMap<String, ClientChannelMap>,
}
impl TopicMap {
    /// Returns the number of connected clients for a given topic.
    pub fn query(&self, topic: String) -> String {
        let v: Vec<String>;
        if topic == "*" {
            v = self
                .map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.len()))
                .collect();
            json!({topic: v}).to_string()
        } else if self.map.contains_key(&topic) {
            v = vec![format!("{}", self.map.get(&topic).unwrap().len())];
            json!({topic: v}).to_string()
        } else {
            "".to_string()
        }
    }
    /// Adds a channel to the map.
    pub fn add_channel(&mut self, topic: String, client_id: String, channel: Sender<Msg>) {
        if self.map.contains_key(&topic.clone()) {
            if let Some(channels) = self.map.get_mut(&topic.clone()) {
                channels.entry(client_id).or_insert(channel);
                // Not sure if the channel should be replaced if the key is already present.
            }
        } else {
            let mut client_map = ClientChannelMap::new();
            client_map.insert(client_id, channel);
            self.map.insert(topic, client_map);
        }
    }
    /// Removes a channel from the map.
    pub fn remove_channel(&mut self, topic: String, client_id: String) {
        if self.map.contains_key(&topic) {
            if let Some(channels) = self.map.get_mut(&topic) {
                channels.remove(&client_id);
            }
            trace!("channels: {:?}", self.map);
        }
    }
    /// adds a topic to the map.
    pub fn add_topic(&mut self, topic: String) {
        self.map.entry(topic).or_default();
    }

    /// Publishes the message to the channels.
    pub async fn publish(&mut self, msg: Msg) {
        if !self.map.contains_key(&msg.topic) {
            return;
        }
        let topic = msg.topic.clone();

        if let Some(channels) = self.map.get_mut(&topic.clone()) {
            let dead_channels = channels
                .iter()
                .map(|(client_id, channel)| {
                    info!("sending msg to the {}", client_id);
                    match channel.send(msg.clone()) {
                        Ok(_n) => "".to_string(),
                        Err(e) => {
                            error!(
                                "error occurred: {} while sending the message to the channel {}",
                                e.to_string(),
                                client_id
                            );
                            error!("cleaning up");
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
    }
}

/// returns a global broadcaster.
pub fn get_global_broadcaster() -> tokio::sync::broadcast::Sender<Msg> {
    info!("creating broadcast channel");
    let (glob_tx, _) = tokio::sync::broadcast::channel(1024);
    glob_tx
}

/// Handles the incoming and out-going messages for each topic.
pub async fn topic_manager(chan: Sender<Msg>) {
    let mut map: TopicMap = TopicMap {
        map: BTreeMap::new(),
    };
    let mut rx = chan.subscribe();
    loop {
        match rx.recv().await {
            Ok(msg) => {
                if !msg.topic.is_empty() {
                    info!("topic received: {}", msg.topic);
                    match msg.header.pkt_type {
                        message::PktType::PUBLISH => {
                            trace!("publishing to map:{:?}", map);
                            map.publish(msg).await;
                        }
                        message::PktType::SUBSCRIBE => {
                            map.add_channel(
                                msg.topic,
                                msg.client_id.unwrap(),
                                msg.channel.unwrap(),
                            );
                            trace!("map: {:?}", map);
                        }
                        message::PktType::UNSUBSCRIBE => {
                            info!("unsubscribing:");
                            map.remove_channel(msg.topic, msg.client_id.unwrap());
                        }
                        message::PktType::QUERY => {
                            info!("querying");
                            let query_resp = map.query(msg.topic.clone());
                            info!("query_resp: {}", query_resp.clone());
                            let resp_msg = match msg.response_msg(query_resp.into_bytes()) {
                                Ok(rm) => rm,
                                Err(e) => {
                                    error!(
                                        "error while getting the response to the query message: {}",
                                        e.to_string()
                                    );
                                    continue;
                                }
                            };
                            info!("generated query resp: {:?}", resp_msg);
                            match msg.channel.unwrap().send(resp_msg) {
                                Ok(n) => n,
                                Err(e) => {
                                    error!(
                                        "error while sending the query response: {}",
                                        e.to_string()
                                    );
                                    0
                                }
                            };
                        }
                        _ => {}
                    };
                }
            }
            Err(e) => {
                error!(
                    "error occurred while receiving the topic: {}",
                    e.to_string()
                );
                // "".to_string()
            }
        };
    }
}
