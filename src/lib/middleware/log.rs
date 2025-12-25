use std::fmt::Debug;

use futures::future::BoxFuture;
use tower::{Layer, Service};

pub struct LogLayer {}

impl<S> Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogService { inner }
    }
}

#[derive(Clone)]
pub struct LogService<S> {
    inner: S,
}

impl<S, Request> Service<Request> for LogService<S>
where
    S: Service<Request>,
    Request: Debug,
    S::Future: Send + 'static,
    S::Response: Debug,
    S::Error: Debug,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        eprintln!("req: {:?}", req);
        let fut = self.inner.call(req);
        Box::pin(async move {
            let res = fut.await;
            eprintln!("resp: {:?}", res);
            res
        })
    }
}
