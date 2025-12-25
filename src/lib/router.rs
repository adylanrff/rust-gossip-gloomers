use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{MaelstromError, Request, Response};

type HandlerFun = fn(r: &Request) -> Result<Response, MaelstromError>;

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

    pub fn handle(&self, r: &Request) -> Result<Response, MaelstromError> {
        let msg_type = r.body().msg_type();

        if let Some(handler) = self.map.read().unwrap().get(&msg_type) {
            return handler(r);
        }

        return Err(MaelstromError::NotSupported);
    }
}
