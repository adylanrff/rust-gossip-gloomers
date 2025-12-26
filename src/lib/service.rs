use std::sync::{Arc, RwLock};

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

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl MessageBody {
    pub fn new(msg_type: String, msg_id: u64, in_reply_to: u64) -> Self {
        Self {
            msg_type,
            msg_id,
            in_reply_to,
            extra: serde_json::Value::default(),
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
    state: Arc<RwLock<NodeState>>,
}

impl MessageContext {
    pub fn new(msg: Message, state: Arc<RwLock<NodeState>>) -> Self {
        Self { msg, state }
    }
}

/*
Error enums based on the Maelstrom protocol
Reference: https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md
*/
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum MaelstromError {
    #[error("timeout")]
    Timeout,

    #[error("node not found")]
    NodeNotFound,

    #[error("not supported")]
    NotSupported,

    #[error("temporarily unavailable")]
    TemporarilyUnavailable,

    #[error("malformed request")]
    MalformedRequest,

    #[error("crash")]
    Crash,

    #[error("Abort")]
    Abort,

    #[error("KeyDoesNotExist")]
    KeyDoesNotExist,

    #[error("KeyAlreadyExist")]
    KeyAlreadyExist,

    #[error("PreconditionFailed")]
    PreconditionFailed,

    #[error("TxnConflict")]
    TxnConflict,
}

impl From<serde_json::Error> for MaelstromError {
    fn from(_: serde_json::Error) -> Self {
        Self::MalformedRequest
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

    type Future = BoxFuture<'static, Result<Self::Response, MaelstromError>>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: MessageContext) -> Self::Future {
        let router = self.router.clone();
        let msg_id = req.msg.body.msg_id;

        Box::pin(async move {
            let res = router.handle(req.msg, req.state).await;
            match res {
                Ok(response) => {
                    let mut msg = response.clone();
                    msg.body.in_reply_to = msg_id;
                    Ok(msg)
                }
                Err(_) => res,
            }
        })
    }
}
