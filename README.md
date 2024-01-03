# Simple-pub-sub

A simple message broker implemented in rust.

The message frame looks like

|header|version|pkt type|topic length|message length|padding|topic|message|
|------|-------|--------|------------|--------------|-------|-----|-------|
|1 byte|2 bytes|1 byte|1 byte|2 bytes|1 byte|.....|.....|

So it's a 8 byte header followed by the topic and message.

## Cli Usage

- Server:
```bash
simple-pub-sub server 0.0.0.0 6480 --log-level trace
```

- Client:
    - subscribe:
    
    ```bash
    simple-pub-sub client 0.0.0.0 6480 subscribe the_topic --log-level trace
    ```
    
    - publish:

    ```bash
    simple-pub-sub client 0.0.0.0 6480 publish the_topic the_message --log-level info
    ```
    
    - query:

    ```bash
    simple-pub-sub client 0.0.0.0 6480 query the_topic --log-level trace
    ```

## API Usage

To subscribe

```rust
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
```
