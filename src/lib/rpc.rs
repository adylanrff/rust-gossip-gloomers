use tokio::sync::mpsc::Sender;

use crate::{MaelstromError, MessageBody};

pub struct Payload {
    dest: String,
    body: MessageBody,
}

#[derive(Debug, Clone)]
pub struct RpcClient {
    chan: Sender<Payload>,
}

impl RpcClient {
    pub fn new(chan: Sender<Payload>) -> Self {
        Self { chan }
    }

    pub fn send(&self, payload: Payload) -> Result<(), MaelstromError> {}
}
