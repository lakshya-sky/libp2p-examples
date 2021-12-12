use super::error::NodeResult;
use tide::prelude::*;

pub struct HttpServer {
    host: String,
    port: u16,
    listener: Option<Box<dyn Listener<()>>>,
}

impl Default for HttpServer {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 8585,
            listener: None,
        }
    }
}

impl HttpServer {
    pub async fn set_listen_addr(&mut self, host: String, port: u16) -> NodeResult<()> {
        self.host = host;
        self.port = port;
        Ok(())
    }

    pub async fn enable(&mut self) -> NodeResult<()> {
        let mut server = tide::new();
        server.at("/").get(|_| async { Ok("Hello, world!") });
        let mut listener = server.bind(format!("{}:{}", self.host, self.port)).await?;
        listener.accept().await?;
        self.listener = Some(Box::new(listener));
        Ok(())
    }

    pub async fn stop(&mut self) {
        match self.listener.as_mut() {
            None => {}
            Some(l) => {
                std::mem::drop(l);
            }
        };
        self.listener = None;
        self.host = "".to_string();
        self.port = 0;
    }
}
