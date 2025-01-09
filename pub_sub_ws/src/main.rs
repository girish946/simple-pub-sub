use std::env;

use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use pub_sub_message::{get_msg_response, msg_from_bytes};

/// Define HTTP actor
struct PubSubWebsocket;

impl Actor for PubSubWebsocket {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PubSubWebsocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                println!("got a ping");
                ctx.pong(&msg)
            }
            Ok(ws::Message::Text(text)) => {
                println!("text: {:?}", text);
                ctx.text(text)
            }
            Ok(ws::Message::Binary(bin)) => {
                println!("text: {:?}", bin);
                println!("parsing the message: {:?}", bin);
                match msg_from_bytes(bin.to_vec()) {
                    Ok(m) => {
                        println!("msg: {:?}", m);
                        let resp = get_msg_response(m).unwrap();
                        ctx.binary(resp)
                    }
                    Err(e) => {
                        println!("unable to parse msg: {}", e);
                        ctx.binary(vec![])
                    }
                };
            }
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(PubSubWebsocket {}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "trace");
    env_logger::init();

    HttpServer::new(|| App::new().route("/ws/", web::get().to(index)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
