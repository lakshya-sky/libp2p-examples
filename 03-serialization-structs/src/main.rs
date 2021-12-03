//Follwing wil create two process, second only subscribed to ws
//cargo run 
//cargo run -- -p 4000 /ip4/127.0.0.1/tcp/3001/ws
mod arguments;
mod behaviour;
mod node;
use futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use std::error::Error;
use structopt::StructOpt;
use tokio::io::{self, AsyncBufReadExt};

use crate::node::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let opt = arguments::Opt::from_args();

    let (local_key, local_peer_id) = generate_identity();
    let mdns = generate_mdns().await?;
    let ws_mdns = generate_mdns().await?;
    let topic = generate_floodsub_topic("chat");
    let transport = config_transport(&local_key);
    let ws_transport = config_ws_transport(&local_key);

    let behaviour = node::floodsub_behaviour(local_peer_id.clone(), mdns, topic.clone()).await;
    let ws_behaviour =
        node::floodsub_behaviour(local_peer_id.clone(), ws_mdns, topic.clone()).await;

    let mut swarm = node::swarm_config(
        opt.dial.clone(),
        opt.port,
        transport,
        TransportType::Tcp,
        behaviour,
        local_peer_id,
    )
    .await?;

    let mut ws_swarm = node::swarm_config(
        opt.dial.clone(),
        opt.port,
        ws_transport,
        TransportType::Ws,
        ws_behaviour,
        local_peer_id,
    )
    .await?;
    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    // Kick it off
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let mut line = line?.expect("stdin closed");
                if let Some(_) = line.find("ws:") {
                    line = line.strip_prefix("ws:").unwrap().to_string();
                    ws_swarm.behaviour_mut().floodsub.publish(topic.clone(), line.as_bytes());
                }else{
                    swarm.behaviour_mut().floodsub.publish(topic.clone(), line.as_bytes());
                }
            }
            event = swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    println!("Listening on {:?}", address);
                }
            }
            ws_event = ws_swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = ws_event {
                    println!("Listening on {:?}", address);
                }
            }
        }
    }
}
