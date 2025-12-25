use std::io::BufReader;

use maelstrom::{self, transport::MaelstromTransport};
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    let router = maelstrom::Router::new().route("echo", |r| {
        println!("whooohoo: {:?}", r);
        Ok(maelstrom::Response {})
    });

    let service = ServiceBuilder::new().service(maelstrom::MaelstromService::new(router));

    let transport = MaelstromTransport::new(service);
    transport.run().await
}
