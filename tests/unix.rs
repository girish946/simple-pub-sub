use tokio::time::{sleep, Duration};
#[cfg(test)]
mod tests {

    use super::*;
    use log::info;
    use simple_pub_sub::server::ServerTrait as _;

    async fn start_serever(addr: String) {
        println!("server started");
        let server = simple_pub_sub::server::ServerType::Unix(simple_pub_sub::server::Unix {
            path: addr.clone(),
        });
        let result = server.start().await;

        info!("{:?}", result);
    }

    #[tokio::test]
    async fn client_publish() {
        // std::env::set_var("RUST_LOG", "trace");
        env_logger::init();

        let path = "/tmp/sample2.sock".to_string();

        let server = tokio::spawn(start_serever(path.clone()));
        sleep(Duration::from_millis(1000)).await;
        let client_type = simple_pub_sub::client::PubSubUnixClient { path };
        // initialize the client.
        let mut client = simple_pub_sub::client::Client::new(
            simple_pub_sub::client::PubSubClient::Unix(client_type),
        );
        // connect the client.
        let _ = client.connect().await;

        // subscribe to the given topic.
        let result = client
            .publish(
                "abc".to_string(),
                "test message".to_string().into_bytes().to_vec(),
            )
            .await;
        info!("{:?}", result);
        std::mem::drop(server);
        sleep(Duration::from_millis(5000)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn client_subscribe() {
        // std::env::set_var("RUST_LOG", "trace");
        let path = "/tmp/sock1.sock".to_string();

        let server = tokio::spawn(start_serever(path.clone()));
        sleep(Duration::from_millis(1000)).await;
        let client_type = simple_pub_sub::client::PubSubUnixClient { path: path.clone() };
        let client_type_pub = simple_pub_sub::client::PubSubUnixClient { path };
        // initialize the client.
        let mut client_sub = simple_pub_sub::client::Client::new(
            simple_pub_sub::client::PubSubClient::Unix(client_type),
        );
        let mut client_pub = simple_pub_sub::client::Client::new(
            simple_pub_sub::client::PubSubClient::Unix(client_type_pub),
        );

        // connect the client.
        let _ = client_sub.connect().await;
        let _ = client_pub.connect().await;
        pub fn on_msg(topic: String, message: Vec<u8>) {
            println!("topic: {} message: {:?}", topic, message);
            assert_eq!(topic, "abc");
        }

        client_sub.on_message(on_msg);
        // connect the client.
        let _ = client_sub.connect().await;
        // subscribe to the given topic.
        let subscribe_client = client_sub.subscribe("abc".to_string());
        let _ = client_pub
            .publish(
                "abc".to_string(),
                "test message".to_string().into_bytes().to_vec(),
            )
            .await;

        std::mem::drop(server);
        std::mem::drop(subscribe_client);
        sleep(Duration::from_millis(5000)).await;
    }
}
