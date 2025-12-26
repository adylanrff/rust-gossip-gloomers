use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{MaelstromError, Message, node::NodeState};

type HandlerFun = fn(r: &Message, node_state: &NodeState) -> Result<Message, MaelstromError>;

#[derive(Default, Clone)]
pub struct Router {
    // "message_type"->fn
    map: Arc<RwLock<HashMap<String, HandlerFun>>>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(self, msg_type: &str, f: HandlerFun) -> Self {
        let cmap = self.map.clone();
        let mut inner = cmap.write().unwrap();
        inner.entry(msg_type.to_string()).or_insert(f);
        self
    }

    pub fn handle(&self, req: &Message, node_state: &NodeState) -> Result<Message, MaelstromError> {
        if let Some(handler) = self.map.read().unwrap().get(&req.body.msg_type) {
            return handler(req, node_state);
        }

        return Err(MaelstromError::NotSupported);
    }
}
