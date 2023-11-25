use log::{error, info};
use std::collections::HashMap;
use tokio;
use tokio::sync::broadcast::Sender;
pub fn get_global_broadcaster() -> tokio::sync::broadcast::Sender<String> {
    info!("creating broadcast channel");
    let (glob_tx, _) = tokio::sync::broadcast::channel(1024);
    glob_tx
}
pub async fn topic_manager(chan: Sender<String>) {
    let mut topics_map: HashMap<String, u16> = HashMap::new();
    let mut rx = chan.subscribe();
    loop {
        let topic = match rx.recv().await {
            Ok(t) => t,
            Err(e) => {
                error!("error occured while receving the topic: {}", e.to_string());
                "".to_string()
            }
        };
        if !topic.is_empty() {
            info!("topic received: {}", topic);
            if topics_map.contains_key(&topic) {
                info!("topic already present: adding the client to it");
                let clients: u16 = match topics_map.get(&topic) {
                    Some(client) => client.clone(),
                    None => 0,
                };
                topics_map.insert(topic, clients);
            } else {
                topics_map.insert(topic, 0);
                info!("new topic, adding it to the hashmap");
            }
            info!("{:?}", topics_map);
        }
    }
}
