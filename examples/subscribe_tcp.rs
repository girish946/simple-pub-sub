use tokio::select;

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
    // subscribe to the given topic.
    client.subscribe("abc".to_string()).await?;

    loop {
        select! {
            msg = client.read_message()=>{
                match  msg{
                    Ok(msg)=>{
                        println!("{}:{:?}", msg.topic, msg.message);
                    }
                    Err(e)=>{
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
    }
}
