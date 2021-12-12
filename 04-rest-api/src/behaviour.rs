use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use serde_json::Value;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct P2PBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for P2PBehaviour {
    // Called when `floodsub` produces an event.
    fn inject_event(&mut self, message: FloodsubEvent) {
        match message {
            FloodsubEvent::Message(message) => {
                let message = String::from_utf8_lossy(&message.data);
                println!("Received: {:?}", message);
                match serde_json::from_str::<Value>(&message) {
                    Ok(v) => {
                        println!("Received: {:?}", v);
                    }
                    Err(_) => {
                        println!("Incorrect format received");
                    }
                }
            }
            _ => {}
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for P2PBehaviour {
    // Called when `mdns` produces an event.
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}
