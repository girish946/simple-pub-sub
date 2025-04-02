use tokio::time::{sleep, Duration};
#[cfg(test)]
mod tests {

    use anyhow::Result;

    use super::*;
    use log::info;
    use simple_pub_sub::server::ServerTrait as _;

    async fn start_serever() -> Result<()> {
        println!("server started");
        let server = simple_pub_sub::server::ServerType::Tcp(simple_pub_sub::server::Tcp {
            host: "localhost".to_string(),
            port: 6480,
            cert: None,
            cert_password: None,
            capacity: 1024,
        });
        server.start().await
    }

    #[tokio::test]
    async fn client_publish() {
        // std::env::set_var("RUST_LOG", "trace");
        env_logger::init();

        let server = tokio::spawn(start_serever());
        sleep(Duration::from_millis(500)).await;
        let client_type = simple_pub_sub::client::PubSubTcpClient {
            server: "localhost".to_string(),
            port: 6480,
            cert: None,
            cert_password: None,
        };
        let callback_fn = |topic: String, message: &[u8]| {
            println!("topic:{:?} message: {:?}", topic, message);
            assert_eq!(topic, "abc");
        };

        // initialize the client.
        let mut client = simple_pub_sub::client::Client::new(
            simple_pub_sub::client::PubSubClient::Tcp(client_type),
            callback_fn,
        );
        // connect the client.
        client.connect().await.unwrap();

        // subscribe to the given topic.
        let result = client
            .publish(
                "abc".to_string(),
                "test message".to_string().into_bytes().to_vec(),
            )
            .await;
        info!("{:?}", result);

        sleep(Duration::from_millis(500)).await;
        assert!(result.is_ok());
        std::mem::drop(server);
    }

    #[tokio::test]
    async fn client_subscribe() {
        // std::env::set_var("RUST_LOG", "trace");

        let server = tokio::spawn(start_serever());
        sleep(Duration::from_millis(500)).await;
        let client_type = simple_pub_sub::client::PubSubTcpClient {
            server: "localhost".to_string(),
            port: 6480,
            cert: None,
            cert_password: None,
        };
        let client_type_pub = simple_pub_sub::client::PubSubTcpClient {
            server: "localhost".to_string(),
            port: 6480,
            cert: None,
            cert_password: None,
        };

        let callback_fn = |topic: String, message: &[u8]| {
            println!("topic:{:?} message: {:?}", topic, message);
            assert_eq!(topic, "abc");
        };

        // initialize the client.
        let mut client_sub = simple_pub_sub::client::Client::new(
            simple_pub_sub::client::PubSubClient::Tcp(client_type),
            callback_fn,
        );
        let mut client_pub = simple_pub_sub::client::Client::new(
            simple_pub_sub::client::PubSubClient::Tcp(client_type_pub),
            callback_fn,
        );

        // connect the client.
        client_sub.connect().await.unwrap();
        client_pub.connect().await.unwrap();

        // connect the client.
        client_sub.connect().await.unwrap();
        // subscribe to the given topic.
        let subscribe_client = client_sub.subscribe("abc".to_string());
        client_pub
            .publish(
                "abc".to_string(),
                "test message".to_string().into_bytes().to_vec(),
            )
            .await
            .unwrap();

        sleep(Duration::from_millis(500)).await;

        std::mem::drop(server);
        std::mem::drop(subscribe_client);
    }
}
