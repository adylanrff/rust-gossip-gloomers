use maelstrom::{self, MaelstromError, Message, node::MaelstromNode};
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    let router = maelstrom::Router::new().route("echo", echo);

    let service = ServiceBuilder::new().service(maelstrom::MaelstromService::new(router));

    let transport = MaelstromNode::new(service);
    transport.run().await
}

fn echo(r: &Message) -> Result<Message, MaelstromError> {
    Ok(Message::new(
        "a".to_string(),
        "b".to_string(),
        r.body.clone(),
    ))
}
