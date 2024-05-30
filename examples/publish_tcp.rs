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
    // connect the client.
    let _ = client.connect().await;
    // subscribe to the given topic.
    let _ = client
        .publish(
            "abc".to_string(),
            "test message".to_string().into_bytes().to_vec(),
        )
        .await;
    Ok(())
}
