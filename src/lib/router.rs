use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{MaelstromError, Message, MessageBody};

type HandlerFun = fn(r: &Message) -> Result<Message, MaelstromError>;

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

    pub fn handle(&self, r: &Message) -> Result<Message, MaelstromError> {
        if let Some(handler) = self.map.read().unwrap().get(&r.body.msg_type) {
            return handler(r);
        }

        return Err(MaelstromError::NotSupported);
    }
}

fn init(r: &Message) -> Result<Message, MaelstromError> {
    let body = MessageBody::new("init_ok".to_string(), 0, r.body.msg_id);
    Ok(Message::new(r.dest.clone(), r.src.clone(), body))
}
