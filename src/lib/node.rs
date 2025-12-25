use std::{
    io::{self, BufWriter, Stdout, Write},
    sync::Arc,
};

use tokio::sync::Mutex;
use tower::Service;

use crate::{MaelstromService, Message};

pub struct MaelstromNode {
    service: MaelstromService,
    writer: Arc<Mutex<BufWriter<Stdout>>>,

    node_id: String,
}

impl MaelstromNode {
    pub fn new(service: MaelstromService) -> Self {
        Self {
            service: service,
            writer: Arc::new(Mutex::new(BufWriter::new(io::stdout()))),
            node_id: String::new(),
        }
    }

    pub async fn run(&self) {
        loop {
            // Read stdin
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            let msg: Message = serde_json::from_str(&buffer).unwrap();

            let mut service = self.service.clone();
            let writer_guard = self.writer.clone();

            tokio::spawn(async move {
                let mut writer = writer_guard.lock().await;

                eprintln!("msg: {:?}", msg);
                let res = service.call(msg).await;
                eprintln!("res: {:?}", res);

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
}
