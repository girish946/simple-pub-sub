#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client_type = simple_pub_sub::client::PubSubTcpClient {
        server: "localhost".to_string(),
        port: 6480,
        cert: None,
        cert_password: None,
    };
    // initialize the client.
    let mut client =
        simple_pub_sub::client::Client::new(simple_pub_sub::client::PubSubClient::Tcp(client_type));

    client.connect().await?;
    // publish to the given topic.
    client
        .publish(
            "abc".to_string(),
            "test message".to_string().into_bytes().to_vec(),
        )
        .await
}
