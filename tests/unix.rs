use tokio::time::{sleep, Duration};
#[cfg(test)]
mod tests {

    use super::*;
    use log::info;
    use simple_pub_sub::server::ServerTrait as _;

    async fn start_serever() {
        let addr = "/tmp/simple.sock".to_string();
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

        let _ = tokio::spawn(start_serever());
        sleep(Duration::from_millis(1000)).await;
        let client_type = simple_pub_sub::client::PubSubUnixClient {
            path: "/tmp/simple.sock".to_string(),
        };
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

        sleep(Duration::from_millis(1000)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn client_subscribe() {
        // std::env::set_var("RUST_LOG", "trace");

        let _ = tokio::spawn(start_serever());
        sleep(Duration::from_millis(1000)).await;
        let client_type = simple_pub_sub::client::PubSubUnixClient {
            path: "/tmp/simple.sock".to_string(),
        };
        let client_type_pub = simple_pub_sub::client::PubSubUnixClient {
            path: "/tmp/simple.sock".to_string(),
        };
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
        let _ = client_sub.subscribe("abc".to_string());
        let _ = client_pub
            .publish(
                "abc".to_string(),
                "test message".to_string().into_bytes().to_vec(),
            )
            .await;

        sleep(Duration::from_millis(1000)).await;
    }
}
