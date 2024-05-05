use simple_pub_sub;
use tokio::time::{sleep, Duration};
#[cfg(test)]
mod tests {

    use super::*;
    use log::info;

    async fn start_serever() {
        let addr = "localhost:6480".to_string();
        println!("server started");
        let _ = simple_pub_sub::server::start_tcp_server(addr).await;
    }

    #[tokio::test]
    async fn client_publish() {
        std::env::set_var("RUST_LOG", "trace");
        env_logger::init();

        let _ = tokio::spawn(start_serever());
        sleep(Duration::from_millis(1000)).await;
        let client_type = simple_pub_sub::client::PubSubTcpClient {
            server: "localhost".to_string(),
            port: 6480,
        };
        // initialize the client.
        let mut client = simple_pub_sub::client::Client::new(
            simple_pub_sub::client::PubSubClient::Tcp(client_type),
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
        assert_eq!(true, result.is_ok());
    }

    #[tokio::test]
    async fn client_subscribe() {
        std::env::set_var("RUST_LOG", "trace");

        let _ = tokio::spawn(start_serever());
        sleep(Duration::from_millis(1000)).await;
        let client_type = simple_pub_sub::client::PubSubTcpClient {
            server: "localhost".to_string(),
            port: 6480,
        };
        let client_type_pub = simple_pub_sub::client::PubSubTcpClient {
            server: "localhost".to_string(),
            port: 6480,
        };

        // initialize the client.
        let mut client_sub = simple_pub_sub::client::Client::new(
            simple_pub_sub::client::PubSubClient::Tcp(client_type),
        );
        let mut client_pub = simple_pub_sub::client::Client::new(
            simple_pub_sub::client::PubSubClient::Tcp(client_type_pub),
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
