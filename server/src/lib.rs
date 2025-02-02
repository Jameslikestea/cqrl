use futures::Future;
use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::Service,
    Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{error::Error, net::SocketAddr, pin::Pin};
use tokio::net::TcpListener;

#[derive(Clone, Copy)]
pub struct Server {
    port: u16,
}

impl Server {
    pub fn new() -> Self {
        Server { port: 8912 }
    }

    pub fn with_port(&mut self, port: u16) {
        self.port = port;
    }

    pub async fn serve(&self) -> Result<(), Box<dyn Error>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;

            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(error) = http1::Builder::new()
                    .serve_connection(io, Handler::new())
                    .await
                {
                    eprintln!("Error serving connection: {:?}", error);
                }
            });
        }
    }
}

struct Handler {}

impl Handler {
    fn new() -> Self {
        Handler {}
    }
}

type Req = Request<Incoming>;

impl Service<Req> for Handler {
    type Response = Response<Full<Bytes>>;

    type Error = hyper::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Req) -> Self::Future {
        let fut = async move {
            println!(
                "Serving request {}",
                req.uri().path_and_query().unwrap().as_str()
            );
            Ok(Response::new(Full::new(Bytes::from("hi, world"))))
        };

        Box::pin(fut)
    }
}
