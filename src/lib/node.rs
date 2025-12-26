use std::{
    fmt::Debug,
    io::{self, BufWriter, Stdout, Write},
    sync::{Arc, RwLock},
};

use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    sync::mpsc::{self, Receiver, Sender},
};
use tower::Service;

use crate::{Message, MessageContext};

const CHANNEL_BUFFER_SIZE: usize = 10;

#[derive(Debug, Clone, Default)]
pub struct NodeState {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub struct MaelstromNode<S> {
    service: S,
}

impl<S> MaelstromNode<S>
where
    S: Service<MessageContext, Response = Message> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Debug + Send + 'static,
{
    pub fn new(service: S) -> Self {
        Self { service: service }
    }

    pub async fn run<R>(&mut self, reader: BufReader<R>)
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        // read -> process -> output
        let (tx_in, rx_in) = mpsc::channel::<Message>(CHANNEL_BUFFER_SIZE);
        let (tx_out, rx_out) = mpsc::channel::<Result<Message, S::Error>>(CHANNEL_BUFFER_SIZE);

        let read_task = tokio::spawn(Self::read(tx_in, reader));
        let process_task = tokio::spawn(Self::process(rx_in, tx_out, self.service.clone()));
        let write_task = tokio::spawn(Self::write(rx_out, BufWriter::new(io::stdout())));
        // 1. threads for reading
        // 2. threads for processing
        // 3. threads for outputing

        let (_, _, _) = tokio::join!(read_task, process_task, write_task,);
    }

    // async fn handle_init(&mut self, r: &Message) {
    //     // self.state.node_id = r.body.node_id.clone();
    //     // self.state.node_ids = r.body.node_ids.clone();
    //     //
    //     let body = MessageBody::new("init_ok".to_string(), 0, r.body.msg_id);
    //     let msg = Message::new(r.dest.clone(), r.src.clone(), body);
    //
    //     let writer_guard = self.writer.clone();
    //     let mut writer = writer_guard.lock().await;
    //
    //     let json_resp = serde_json::to_string(&msg).unwrap();
    //     writeln!(writer, "{}", json_resp).unwrap();
    //     writer.flush().unwrap();
    // }
    //
    async fn read<R>(sender: Sender<Message>, reader: BufReader<R>)
    where
        R: AsyncRead + Unpin,
    {
        let mut lines = reader.lines();
        loop {
            let line = lines.next_line().await.unwrap().unwrap();
            let req: Message = serde_json::from_str(&line).unwrap();
            sender.send(req).await.unwrap();
        }
    }

    async fn process(
        mut rx_in: Receiver<Message>,
        tx_out: Sender<Result<Message, S::Error>>,
        service: S,
    ) -> Result<(), anyhow::Error> {
        // Init node state
        let node_state: Arc<RwLock<NodeState>> = Arc::default();

        while let Some(msg) = rx_in.recv().await {
            let mut service = service.clone();
            let node_state = node_state.clone();
            let tx_out = tx_out.clone();

            tokio::spawn(async move {
                let msg_ctx = MessageContext::new(msg, node_state);
                let res = service.call(msg_ctx).await;
                tx_out.send(res).await.unwrap();
            });
        }

        Ok(())
    }

    async fn write(
        mut rx_out: Receiver<Result<Message, S::Error>>,
        mut writer: BufWriter<Stdout>,
    ) -> Result<(), anyhow::Error> {
        // Write to stdout
        while let Some(res) = rx_out.recv().await {
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
        }
        Ok(())
    }
}
