// define the on_message function (callback).
pub fn on_msg(topic: String, message: Vec<u8>) {
    println!("topic: {} message: {:?}", topic, message)
}
#[tokio::main]
async fn main() -> Result<(), String> {
    let client_type = simple_pub_sub::client::PubSubTcpClient {
        server: "localhost".to_string(),
        port: 6480,
        cert: None,
        cert_password: None,
    };

    let tls_client = simple_pub_sub::client::PubSubTcpClient {
        server: "localhost".to_string(),
        port: 6480,
        cert: Some("cert.pem".to_string()),
        cert_password: Some("password".to_string()),
    };

    // initialize the client.
    let mut client =
        simple_pub_sub::client::Client::new(simple_pub_sub::client::PubSubClient::Tcp(client_type));
    // set the callback function.
    client.on_message(on_msg);
    // connect the client.
    let _ = client.connect().await;
    // subscribe to the given topic.
    let _ = client.subscribe("abc".to_string()).await;
    Ok(())
}
