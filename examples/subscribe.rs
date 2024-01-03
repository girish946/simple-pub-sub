// use simple-pub-sub
use simple_pub_sub;

// define the on_message function (callback).
pub fn on_msg(topic: String, message: Vec<u8>) {
    println!("topic: {} message: {:?}", topic, message)
}
#[tokio::main]
async fn main() -> Result<(), String> {
    // initialize the client.
    let mut client = simple_pub_sub::client::Client::new("localhost".to_string(), 6480);
    // set the callback function.
    client.on_message(on_msg);
    // connect the client.
    let _ = client.connect().await;
    // subscribe to the given topic.
    let _ = client.subscribe("abc".to_string()).await;
    Ok(())
}
