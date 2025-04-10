# Simple-pub-sub

A simple message broker implemented in rust.

The message frame looks like

|header|version|pkt type|topic length|message length|padding|topic|message|
|------|-------|--------|------------|--------------|-------|-----|-------|
|1 byte|2 bytes|1 byte|1 byte|2 bytes|1 byte|.....|.....|

So it's a 8 byte header followed by the topic and message.

## API Usage

To subscribe

### Client

```rust
use simple-pub-sub

// define the on_message function (callback).
pub fn on_msg(topic: String, message: Vec<u8>) {
    println!("topic: {} message: {:?}", topic, message)
}
#[tokio::main]
async fn main() -> Result<(), String> {
    let client_type = simple_pub_sub::client::PubSubTcpClient {
        server: "localhost".to_string(),
        port: 6480,
    };
    // initialize the client.
    let mut client =
        simple_pub_sub::client::Client::new(simple_pub_sub::client::PubSubClient::Tcp(client_type));
    // connect the client.
    client.connect().await.unwrap();
    // subscribe to the given topic.
    client.subscribe("abc".to_string()).await.unwrap();
    loop{
        let msg = client.read_message().await().unwrap();
        println!("{}: {:?}", msg.topic, msg.message)
    }

    Ok(())
}
```

To publish a message

```rust
use simple_pub_sub;
async fn main() -> Result<(), String> {
    let client_type = simple_pub_sub::client::PubSubTcpClient {
        server: "localhost".to_string(),
        port: 6480,
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
```

### Server

```rust
use simple_pub_sub;
async fn main(){
  let server = simple_pub_sub::server::ServerType::Tcp(
      simple_pub_sub::server::Tcp {
        host: "localhost".to_string(),
        port: 6480,
        cert: None,
        cert_password: None,
        capacity: 1024,
      });
  server.start().await;
}
```

## Cli Usage

- Server:

  - Using Tcp socket:

    ```bash
    simple-pub-sub server tcp 0.0.0.0 6480 --log-level trace
    ```

    To use TLS:

    ```bash
    # Generate the server key and certificate
    openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem \
        -days 365 -nodes -subj "/CN=localhost"
    # generate the identity file.
    openssl pkcs12 -export -out identity.pfx -inkey key.pem -in cert.pem\
      -passout pass:password
    ```

    Start the server using these keys:

    ```bash
    simple-pub-sub server tcp -c identity.pfx -p password 0.0.0.0 6480 \
      --log-level trace
    ```

  - Using Unix socket:

    ```bash
    simple-pub-sub server unix /tmp/pubsub.sock --log-level trace
    ```

- Client:
  - Using Tcp socket:
    - subscribe:

        ```bash
        simple-pub-sub client subscribe the_topic tcp 0.0.0.0 6480 --log-level trace
        ```

        Using TLS:

        ```bash
        simple-pub-sub client subscribe the_topic tcp -c certs/cert.pem 0.0.0.0 6480\
            --log-level trace
        ```

    - publish:

        ```bash
        simple-pub-sub client publish the_topic the_message tcp 0.0.0.0 6480\
            --log-level info
        ```

        Using TLS:

        ```bash
        simple-pub-sub client publish the_topic the_message tcp -c certs/cert.pem\
         0.0.0.0 6480 --log-level trace
        ```

    - query:

        ```bash
        simple-pub-sub client query the_topic tcp 0.0.0.0 6480  --log-level trace
        ```

        Using TLS:

        ```bash
        simple-pub-sub client query the_topic tcp -c certs/cert.pem\
          0.0.0.0 6480  --log-level trace
        ```

  - Using Unix socket:
    - subscribe:

        ```bash
        simple-pub-sub client unix /tmp/pubsub.sock subscribe the_topic\
          --log-level trace
        ```

    - publish:

        ```bash
        simple-pub-sub client unix /tmp/pubsub.sock publish the_topic the_message\
          --log-level info
        ```

    - query:

        ```bash
        simple-pub-sub client unix /tmp/pubsub.sock query the_topic --log-level trace
        ```

### Shell completion

Supported shells: `bash`, `zsh`, `fish`, `Elvish`, `Powershell`
To add the shell completions, run

```bash
simple-pub-sub completion <shell> 
```

To add the shell completions to `.bashrc`

```bash
source <(simple-pub-sub completion bash) >> ~/.bashrc
```
