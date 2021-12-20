use std::{convert::Infallible, net::SocketAddr};

use super::error::NodeResult;
use hyper::{
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Request, Response,
};
use tokio::sync::oneshot::{self, Receiver, Sender};

pub struct HttpServer {
    host: String,
    port: u16,
    signal_reciever: Receiver<()>,
    stop_sender: Sender<()>,
}

impl HttpServer {
    pub fn new(host: String, port: u16) -> Self {
        let (tx, rx) = oneshot::channel();
        HttpServer {
            host,
            port,
            stop_sender: tx,
            signal_reciever: rx,
        }
    }

    pub async fn set_listen_addr(&mut self, host: String, port: u16) -> NodeResult<()> {
        self.host = host;
        self.port = port;
        Ok(())
    }

    pub async fn enable(&mut self) -> NodeResult<()> {
        let make_service =
            make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(hello_world)) });
        let socker_addr: SocketAddr =
            String::from(format!("{}:{}", self.host, self.port)).parse()?;
        let server = Server::bind(&socker_addr).serve(make_service);
        let server = server.with_graceful_shutdown(self.handle_shutdown());
        Ok(server.await?)
    }

    async fn handle_shutdown(&mut self) {
        let t = &mut self.signal_reciever;
        t.await.ok();
    }

    pub async fn stop(&mut self) {
        self.host = "".to_string();
        self.port = 0;
        self.handle_shutdown().await;
        println!("Successfully shutdown!");
    }
}

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
}
