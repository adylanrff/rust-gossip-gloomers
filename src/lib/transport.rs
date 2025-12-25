use std::{
    io::{self, BufWriter, Stdout, Write},
    sync::{Arc, Mutex},
};

use tower::Service;

use crate::{MaelstromService, Request};

pub struct MaelstromTransport {
    service: MaelstromService,
    writer: Arc<Mutex<BufWriter<Stdout>>>,
}

impl MaelstromTransport {
    pub fn new(service: MaelstromService) -> Self {
        Self {
            service: service,
            writer: Arc::new(Mutex::new(BufWriter::new(io::stdout()))),
        }
    }

    pub async fn run(&self) {
        loop {
            // Read STDIN
            let req: Request = Request::default();
            let mut service = self.service.clone();
            let writer_guard = self.writer.clone();

            tokio::spawn(async move {
                let res = service.call(req).await;
                let mut writer = writer_guard.lock().unwrap();
                match res {
                    Ok(response) => {
                        let json_resp = serde_json::to_string(&response).unwrap();
                        writer.write(json_resp.as_bytes()).unwrap();
                        writer.flush().unwrap();
                    }
                    Err(err) => {}
                };
            })
            .await
            .expect("error");
        }
    }
}
