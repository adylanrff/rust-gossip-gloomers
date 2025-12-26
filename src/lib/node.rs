use std::{
    fmt::Debug,
    io::{self, BufWriter, Stdout, Write},
    sync::Arc,
};

use futures::FutureExt;
use serde::Serialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    sync::Mutex,
};
use tower::Service;

use crate::{Message, MessageBody, MessageContext};

#[derive(Debug, Clone, Default)]
pub struct NodeState {
    node_id: String,
    node_ids: Vec<String>,
}

pub struct MaelstromNode<S> {
    service: S,
    writer: Arc<Mutex<BufWriter<Stdout>>>,

    state: NodeState,
}

impl<S> MaelstromNode<S>
where
    S: Service<MessageContext> + Clone + Send + 'static,
    S::Response: Serialize,
    S::Future: Send + 'static,
    S::Error: Debug,
{
    pub fn new(service: S) -> Self {
        Self {
            service: service,
            writer: Arc::new(Mutex::new(BufWriter::new(io::stdout()))),
            state: NodeState::default(),
        }
    }

    pub async fn run<R>(&mut self, reader: BufReader<R>)
    where
        R: AsyncRead + Unpin,
    {
        let mut lines = reader.lines();
        loop {
            // Read stdin
            let line = lines.next_line().fuse().await.unwrap().unwrap();
            let req: Message = serde_json::from_str(&line).unwrap();

            let mut service = self.service.clone();
            let writer_guard = self.writer.clone();

            let msg = MessageContext::new(req.clone(), self.state.clone());

            if req.body.msg_type == "init" {
                self.handle_init(&req).await;
                continue;
            }

            tokio::spawn(async move {
                let mut writer = writer_guard.lock().await;
                let res = service.call(msg).await;

                // Write to stdout
                match res {
                    Ok(response) => {
                        let json_resp = serde_json::to_string(&response).unwrap();
                        writeln!(writer, "{}", json_resp).unwrap();
                        writer.flush().unwrap();
                    }
                    Err(err) => {
                        eprintln!("err: {:?}", err)
                    }
                };
            })
            .await
            .expect("error");
        }
    }

    async fn handle_init(&mut self, r: &Message) {
        self.state.node_id = r.body.node_id.clone();
        self.state.node_ids = r.body.node_ids.clone();

        let body = MessageBody::new("init_ok".to_string(), 0, r.body.msg_id);
        let msg = Message::new(r.dest.clone(), r.src.clone(), body);

        let writer_guard = self.writer.clone();
        let mut writer = writer_guard.lock().await;

        let json_resp = serde_json::to_string(&msg).unwrap();
        writeln!(writer, "{}", json_resp).unwrap();
        writer.flush().unwrap();
    }
}
