use std::{borrow::Cow, fmt::Display};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::task::Poll;

use futures::future::BoxFuture;
use tower::Service;

use crate::Router;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MessageBody {
    msg_type: String,
    msg_id: u64,
    in_reply_to: u64,
}

impl MessageBody {
    pub fn msg_type(&self) -> String {
        self.msg_type.clone()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Request {
    src: String,
    dest: String,
    body: MessageBody,
}

impl Request {
    pub fn body(&self) -> MessageBody {
        MessageBody {
            msg_type: self.body.msg_type.clone(),
            msg_id: self.body.msg_id,
            in_reply_to: self.body.in_reply_to,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {}

#[derive(Debug, Error, Serialize, Deserialize)]

/*
Error enums based on the Maelstrom protocol
Reference: https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md
*/
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

impl Service<Request> for MaelstromService {
    type Response = Response;

    type Error = MaelstromError;

    type Future = BoxFuture<'static, Result<Response, MaelstromError>>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let res = self.router.handle(&req);
        Box::pin(async { res })
    }
}
