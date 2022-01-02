use std::net::SocketAddr;

use super::error::NodeResult;
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_http_server::{
    jsonrpc_core::{IoHandler, Params, Value},
    Server,
};

pub struct HttpServer {
    internal_server: Option<Server>,
    host: String,
    port: u16,
}

impl HttpServer {
    pub fn new(host: String, port: u16) -> Self {
        HttpServer {
            internal_server: None,
            host,
            port,
        }
    }

    pub async fn set_listen_addr(&mut self, host: String, port: u16) -> NodeResult<()> {
        self.host = host;
        self.port = port;
        Ok(())
    }

    pub async fn enable(&mut self) -> NodeResult<()> {
        let socker_addr: SocketAddr =
            String::from(format!("{}:{}", self.host, self.port)).parse()?;
        let mut io = IoHandler::default();
        io.add_method("hello", |_params: Params| async {
            Ok(Value::String("hello".to_owned()))
        });
        let server = ServerBuilder::new(io)
            .threads(1)
            .start_http(&socker_addr)
            .unwrap();
        self.internal_server = Some(server);
        Ok(())
    }

    pub async fn stop(&mut self) {
        self.host = "".to_string();
        self.port = 0;
        let internal_server = self.internal_server.take();
        tokio::task::spawn_blocking(|| drop(internal_server));
        println!("Successfully shutdown!");
    }
}
