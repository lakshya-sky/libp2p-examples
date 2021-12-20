use crate::{
    node::{Node, NodeConfig},
    p2p::P2PConfigBuilder,
};
use async_std::task;
use libp2p::Multiaddr;
use std::{error::Error, time::Duration};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opt {
    pub peer: Option<Multiaddr>,
    #[structopt(long = "http-host", default_value = "127.0.0.1")]
    pub http_host: String,
    #[structopt(long = "http-port", default_value = "8585")]
    pub http_port: u16,
    #[structopt(long = "p2p-port", default_value = "8500")]
    pub p2p_port: u16,
}

fn node_config_from_args(opt: &Opt) -> NodeConfig {
    let mut p2p_config = P2PConfigBuilder::default().set_port(opt.p2p_port).build();
    if let Some(p) = &opt.peer {
        p2p_config.add_peer(p.clone());
    }
    NodeConfig::new(
        opt.http_host.clone(),
        opt.http_port,
        p2p_config, //p2p_config
    )
}

pub async fn init_using_args() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let config = node_config_from_args(&opt);
    let mut node = Node::new(config)?;
    node.start().await?;
    println!("Node running!");
    task::sleep(Duration::from_millis(5000)).await;
    node.stop().await;
    Ok(())
}
