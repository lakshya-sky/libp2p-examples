use std::sync::Arc;

use crate::p2p::{P2PConfig, P2PServer};
mod error;
mod rpc_server;
use error::*;
use rpc_server::*;
use tokio::sync::{oneshot, Mutex};

enum NodeSignal {
    StopHttp,
}

pub enum TransportType {
    Tcp,
    Ws,
}

pub struct NodeConfig {
    pub http_host: String,
    pub http_port: u16,
    pub p2p: P2PConfig,
}

impl NodeConfig {
    pub fn new(http_host: String, http_port: u16, p2p_config: P2PConfig) -> Self {
        NodeConfig {
            http_host,
            http_port,
            p2p: p2p_config,
        }
    }
}

#[derive(Debug)]
enum NodeState {
    Init,
    Running,
    Closed,
}

pub struct Node {
    config: NodeConfig,
    server: P2PServer,
    http: HttpServer,
    state: NodeState,
}

impl Node {
    pub fn new(config: NodeConfig) -> NodeResult<Self> {
        let http = HttpServer::new(
            config.http_host.clone(),
            config.http_port,
        );
        Ok(Node {
            http,
            server: P2PServer::new(config.p2p.clone())?,
            config,
            state: NodeState::Init,
        })
    }

    pub async fn start(&mut self) -> NodeResult<()> {
        match self.state {
            NodeState::Running => Err(Box::new(NodeError::NodeRunning)),
            NodeState::Closed => Err(Box::new(NodeError::NodeStopped)),
            NodeState::Init => {
                self.state = NodeState::Running;
                self.open_end_points().await?;
                Ok(())
            }
        }
    }

    pub async fn stop(&mut self) {
        match self.state {
            NodeState::Init => {}
            NodeState::Running => {
                self.stop_rpc().await;
            }
            NodeState::Closed => {}
        }
        self.state = NodeState::Closed;
    }

    async fn open_end_points(&mut self) -> NodeResult<()> {
        println!("p2p starting");
        self.server.start().await?;
        self.start_rpc().await?;
        Ok(())
    }

    async fn start_rpc(&mut self) -> NodeResult<()> {
        self.http
            .set_listen_addr(self.config.http_host.clone(), self.config.http_port)
            .await?;
        self.http.enable().await?;
        Ok(())
    }

    async fn stop_rpc(&mut self) {
        self.http.stop().await;
    }
}
