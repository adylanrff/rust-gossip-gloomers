use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::task::Poll;

use futures::future::BoxFuture;
use tower::Service;

use crate::{Router, node::NodeState};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct MessageBody {
    #[serde(rename = "type")]
    pub msg_type: String,

    pub msg_id: u64,

    pub in_reply_to: u64,

    // Initializations
    pub node_id: String,
    pub node_ids: Vec<String>,
}

impl MessageBody {
    pub fn new(msg_type: String, msg_id: u64, in_reply_to: u64) -> Self {
        Self {
            msg_type,
            msg_id,
            in_reply_to,
            node_id: String::new(),
            node_ids: vec![],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: MessageBody,
}

impl Message {
    pub fn new(src: String, dest: String, body: MessageBody) -> Self {
        Self { src, dest, body }
    }
}

#[derive(Debug, Clone)]
pub struct MessageContext {
    msg: Message,
    state: NodeState,
}

impl MessageContext {
    pub fn new(msg: Message, state: NodeState) -> Self {
        Self { msg, state }
    }
}

/*
Error enums based on the Maelstrom protocol
Reference: https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md
*/
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum MaelstromError {
    Timeout,

    NodeNotFound,

    NotSupported,

    TemporarilyUnavailable,

    MalformedRequest,

    Crash,

    Abort,

    KeyDoesNotExist,

    KeyAlreadyExist,

    PreconditionFailed,

    TxnConflict,
}

impl Display for MaelstromError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Default, Clone)]
pub struct MaelstromService {
    router: Router,
}

impl MaelstromService {
    pub fn new(router: Router) -> Self {
        Self { router }
    }
}

impl Service<MessageContext> for MaelstromService {
    type Response = Message;

    type Error = MaelstromError;

    type Future = BoxFuture<'static, Result<Message, MaelstromError>>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: MessageContext) -> Self::Future {
        let res = self.router.handle(&req.msg, &req.state);
        Box::pin(async { res })
    }
}
