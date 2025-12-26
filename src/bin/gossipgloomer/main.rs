use std::sync::{Arc, RwLock};

use maelstrom::{
    self, MaelstromError, Message,
    middleware::log::LogLayer,
    node::{MaelstromNode, NodeState},
};
use tokio::io::{BufReader, stdin};
use tower::ServiceBuilder;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let router = maelstrom::Router::new()
        .route("echo", echo)
        .route("generate", generate);
    let service = ServiceBuilder::new()
        .layer(LogLayer {})
        .service(maelstrom::MaelstromService::new(router));

    let mut transport = MaelstromNode::new(service);

    let reader = BufReader::new(stdin());
    transport.run(reader).await;
}

fn echo(r: Message, _: Arc<RwLock<NodeState>>) -> Result<Message, MaelstromError> {
    let mut msg_body = r.body.clone();
    msg_body.msg_type = "echo_ok".to_string();

    Ok(Message::new(
        r.dest.to_string(),
        r.src.to_string(),
        msg_body,
    ))
}

fn generate(r: Message, _: Arc<RwLock<NodeState>>) -> Result<Message, MaelstromError> {
    let mut msg_body = r.body.clone();

    msg_body.msg_type = "generate_ok".to_string();
    msg_body.extra["id"] = serde_json::Value::String(Uuid::new_v4().to_string());

    Ok(Message::new(
        r.dest.to_string(),
        r.src.to_string(),
        msg_body,
    ))
}
