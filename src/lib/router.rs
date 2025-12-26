use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{MaelstromError, Message, node::NodeState};

type HandlerFun =
    fn(r: Message, node_state: Arc<RwLock<NodeState>>) -> Result<Message, MaelstromError>;

#[derive(Default, Clone)]
pub struct Router {
    // "message_type"->fn
    map: Arc<RwLock<HashMap<String, HandlerFun>>>,
}

impl Router {
    pub fn new() -> Self {
        Self::default().route("init", init)
    }

    pub fn route(self, msg_type: &str, f: HandlerFun) -> Self {
        let cmap = self.map.clone();
        let mut inner = cmap.write().unwrap();
        inner.entry(msg_type.to_string()).or_insert(f);
        self
    }

    pub async fn handle(
        &self,
        req: Message,
        node_state: Arc<RwLock<NodeState>>,
    ) -> Result<Message, MaelstromError> {
        if let Some(handler) = self.map.read().unwrap().get(&req.body.msg_type) {
            return handler(req, node_state);
        }

        return Err(MaelstromError::NotSupported);
    }
}

fn init(r: Message, node_state: Arc<RwLock<NodeState>>) -> Result<Message, MaelstromError> {
    let mut node_state = node_state.write().unwrap();

    node_state.node_id = serde_json::from_value(r.body.extra["node_id"].clone())?;
    node_state.node_ids = serde_json::from_value(r.body.extra["node_ids"].clone())?;

    let body = crate::MessageBody::new("init_ok".to_string(), 0, r.body.msg_id);
    let msg = Message::new(r.dest.clone(), r.src.clone(), body);
    Ok(msg)
}
