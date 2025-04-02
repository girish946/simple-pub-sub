#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client_type = simple_pub_sub::client::PubSubUnixClient {
        path: "/tmp/simple.sock".to_string(),
    };
    // initialize the client.
    let mut client = simple_pub_sub::client::Client::new(
        simple_pub_sub::client::PubSubClient::Unix(client_type),
        |topic, message| {
            println!("topic:{:?} message: {:?}", topic, message);
        },
    );

    client.connect().await?;
    // publish to the given topic.
    client
        .publish(
            "abc".to_string(),
            "test message".to_string().into_bytes().to_vec(),
        )
        .await
}
