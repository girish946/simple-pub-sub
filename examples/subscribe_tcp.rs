#[tokio::main]
async fn main() -> Result<(), String> {
    let client_type = simple_pub_sub::client::PubSubTcpClient {
        server: "localhost".to_string(),
        port: 6480,
        cert: None,
        cert_password: None,
    };

    // initialize the client.
    let mut client =
        simple_pub_sub::client::Client::new(simple_pub_sub::client::PubSubClient::Tcp(client_type));

    let on_msg = |topic: String, message: &[u8]| {
        println!("topic: {} message: {:?}", topic, message);
        assert_eq!(topic, "abc");
    };

    // connect the client.
    let _ = client.connect().await;
    // subscribe to the given topic.
    let _ = client.subscribe("abc".to_string(), on_msg).await;
    Ok(())
}
