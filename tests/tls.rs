use tokio::time::{sleep, Duration};
async fn create_tls_certs() {
    use std::process::Command;
    // openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes -subj "/CN=localhost"
    // openssl pkcs12 -export -out identity.pfx -inkey key.pem -in cert.pem -passout pass:password

    let cert_gen_op = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:4096",
            "-keyout",
            "certs/key.pem",
            "-out",
            "certs/cert.pem",
            "-days",
            "365",
            "-nodes",
            "-subj",
            "/CN=localhost",
        ])
        .output();
    println!("certs created: {:?}", cert_gen_op);
    let op = Command::new("openssl")
        .args([
            "pkcs12",
            "-export",
            "-out",
            "certs/identity.pfx",
            "-inkey",
            "certs/key.pem",
            "-in",
            "certs/cert.pem",
            "-passout",
            "pass:password",
        ])
        .output();
    println!("identity file created: {:?}", op);
}
#[cfg(test)]
mod tests {

    use super::*;
    use log::info;

    async fn start_serever() {
        let host = "0.0.0.0".to_string();
        let port = 6481;
        let cert = "certs/identity.pfx".to_string();
        let password = "password".to_string();

        println!("server started");
        let server = simple_pub_sub::server::Server {
            server_type: simple_pub_sub::server::ServerType::Tcp(simple_pub_sub::server::Tcp {
                host: host.clone(),
                port,
                cert: Some(cert.clone()),
                cert_password: Some(password.clone()),
            }),
        };
        let _ = server.start().await;
    }

    #[tokio::test]
    async fn tls_client_publish() {
        // std::env::set_var("RUST_LOG", "trace");
        env_logger::init();
        create_tls_certs().await;
        sleep(Duration::from_millis(5000)).await;

        let server = tokio::spawn(start_serever());
        sleep(Duration::from_millis(1000)).await;
        let client_type = simple_pub_sub::client::PubSubTcpClient {
            server: "localhost".to_string(),
            port: 6481,
            cert: Some("certs/cert.pem".to_string()),
            cert_password: Some("password".to_string()),
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
        assert!(result.is_ok());
        std::mem::drop(server);
    }
    #[tokio::test]
    async fn tls_client_subscribe() {
        // std::env::set_var("RUST_LOG", "trace");

        create_tls_certs().await;
        sleep(Duration::from_millis(5000)).await;

        let server = tokio::spawn(start_serever());
        sleep(Duration::from_millis(1000)).await;
        let client_type = simple_pub_sub::client::PubSubTcpClient {
            server: "localhost".to_string(),
            port: 6481,
            cert: Some("certs/cert.pem".to_string()),
            cert_password: Some("password".to_string()),
        };
        let client_type_pub = simple_pub_sub::client::PubSubTcpClient {
            server: "localhost".to_string(),
            port: 6481,
            cert: Some("certs/cert.pem".to_string()),
            cert_password: Some("password".to_string()),
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
        let subscribe_client = client_sub.subscribe("abc".to_string());
        let _ = client_pub
            .publish(
                "abc".to_string(),
                "test message".to_string().into_bytes().to_vec(),
            )
            .await;

        sleep(Duration::from_millis(1000)).await;
        std::mem::drop(server);
        std::mem::drop(subscribe_client);
    }
}
