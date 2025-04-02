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

    // connect the client.
    client.connect().await?;
    // subscribe to the given topic.
    client.subscribe("abc".to_string()).await?;
    client.run().await
}
