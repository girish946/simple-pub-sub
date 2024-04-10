use simple_pub_sub;

#[tokio::main]
async fn main() -> Result<(), String> {
    // initialize the client.
    let mut client = simple_pub_sub::client::Client::new("localhost".to_string(), 6480);
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
