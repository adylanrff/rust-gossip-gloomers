use maelstrom::{
    self, MaelstromError, Message,
    middleware::log::LogLayer,
    node::{MaelstromNode, NodeState},
};
use tokio::io::{BufReader, stdin};
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    let router = maelstrom::Router::new().route("echo", echo);
    let service = ServiceBuilder::new()
        .layer(LogLayer {})
        .service(maelstrom::MaelstromService::new(router));

    let mut transport = MaelstromNode::new(service);

    let reader = BufReader::new(stdin());
    transport.run(reader).await;
}

fn echo(r: &Message, _: &NodeState) -> Result<Message, MaelstromError> {
    Ok(Message::new(
        "a".to_string(),
        "b".to_string(),
        r.body.clone(),
    ))
}
