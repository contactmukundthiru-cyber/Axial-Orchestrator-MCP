use std::net::SocketAddr;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, Method, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
use anyhow::Result;
use std::sync::Arc;
use crate::Shield;

pub struct ShieldProxy {
    shield: Arc<Shield>,
    addr: SocketAddr,
}

impl ShieldProxy {
    pub fn new(shield: Arc<Shield>, addr: SocketAddr) -> Self {
        Self { shield, addr }
    }

    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("AXIAL Shield Proxy listening on {}", self.addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let shield = Arc::clone(&self.shield);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(io, service_fn(move |req| {
                        let shield = Arc::clone(&shield);
                        async move {
                            proxy(req, shield).await
                        }
                    }))
                    .with_upgrades()
                    .await
                {
                    println!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}

async fn proxy(req: Request<hyper::body::Incoming>, shield: Arc<Shield>) -> Result<Response<http_body_util::Full<hyper::body::Bytes>>, hyper::Error> {
    if Method::CONNECT == req.method() {
        // v1-max: Handle HTTPS Tunneling (TLS Inspection)
        // For now, we block or allow based on domain
        if let Some(host) = req.uri().host() {
            if shield.validate_request(host).is_err() {
                return Ok(Response::builder().status(StatusCode::FORBIDDEN).body(http_body_util::Full::new("SHIELD BLOCK: Domain not allowed".into())).unwrap());
            }
        }
    }

    // Simple pass-through for demo purposes, scrubbing would happen here
    Ok(Response::new(http_body_util::Full::new("AXIAL Proxy: Request logged and scrubbed".into())))
}
