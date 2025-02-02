use futures::Future;
use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::Service,
    Request, Response,
};
use hyper_util::rt::TokioIo;
use parser::API;
use std::{error::Error, net::SocketAddr, pin::Pin};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Server {
    port: u16,
    api: API,
}

impl Server {
    pub fn new() -> Self {
        Server {
            port: 8912,
            api: API::new(),
        }
    }

    pub fn with_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn with_api(&mut self, api: API) {
        self.api = api;
    }

    pub async fn serve(&self) -> Result<(), Box<dyn Error>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener = TcpListener::bind(addr).await?;
        let api = self.api.clone();

        for command in api.commands.iter() {
            println!("Discovered Command: {}", command.name);
        }
        for query in api.queries.iter() {
            println!("Discoviered Query: {}", query.name);
        }

        loop {
            let handler = Handler::new(api.clone());
            let (stream, _) = listener.accept().await?;

            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(error) = http1::Builder::new().serve_connection(io, handler).await {
                    eprintln!("Error serving connection: {:?}", error);
                }
            });
        }
    }
}

struct Handler<'a> {
    _name: &'a str,
    _api: API,
}

impl<'a> Handler<'a> {
    fn new(api: API) -> Self {
        Handler {
            _name: "handler",
            _api: api,
        }
    }
}

type Req = Request<Incoming>;

impl<'a> Service<Req> for Handler<'a> {
    type Response = Response<Full<Bytes>>;

    type Error = hyper::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'a>>;

    fn call(&self, req: Req) -> Self::Future {
        let fut = async move {
            match req.uri().path() {
                "/command." => {
                    println!("Running Command");
                }
                "/query." => {
                    println!("Running Query");
                }
                path => {
                    println!("Running Path: {}", path);
                }
            }

            Ok(Response::new(Full::new(Bytes::from("hi, world"))))
        };

        Box::pin(fut)
    }
}
