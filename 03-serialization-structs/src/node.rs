use crate::behaviour::MyBehaviour;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed, upgrade},
    floodsub::{self, Floodsub},
    identity,
    mdns::Mdns,
    mplex, noise,
    swarm::SwarmBuilder,
    tcp::TokioTcpConfig,
    websocket::WsConfig,
    Multiaddr, PeerId, Swarm, Transport,
};
use std::error::Error;

pub enum TransportType {
    Tcp,
    Ws,
}

pub fn generate_identity() -> (identity::Keypair, PeerId) {
    let local_key: identity::Keypair = identity::Keypair::generate_ed25519();
    let local_peer_id: PeerId = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);
    (local_key, local_peer_id)
}

pub fn config_transport(local_key: &identity::Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(local_key)
        .expect("Signing libp2p-noise static DH keypair failed.");
    let transport = TokioTcpConfig::new()
        .nodelay(true)
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();
    transport
}

pub fn config_ws_transport(local_key: &identity::Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(local_key)
        .expect("Signing libp2p-noise static DH keypair failed.");
    let transport = TokioTcpConfig::new().nodelay(true);
    let ws_transport = WsConfig::new(transport)
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();
    ws_transport
}

pub fn generate_floodsub_topic(name: &str) -> floodsub::Topic {
    floodsub::Topic::new(name)
}

pub async fn generate_mdns() -> Result<Mdns, Box<dyn Error>> {
    Ok(Mdns::new(Default::default()).await?)
}

pub async fn floodsub_behaviour(
    local_peer_id: PeerId,
    mdns: Mdns,
    topic: floodsub::Topic,
) -> MyBehaviour {
    let mut behaviour = MyBehaviour {
        floodsub: Floodsub::new(local_peer_id.clone()),
        mdns,
    };
    behaviour.floodsub.subscribe(topic.clone());
    behaviour
}

pub async fn swarm_config(
    dial: Option<Multiaddr>,
    listen_on: u16,
    transport: Boxed<(PeerId, StreamMuxerBox)>,
    transport_type: TransportType,
    behaviour: MyBehaviour,
    local_peer_id: PeerId,
) -> Result<Swarm<MyBehaviour>, Box<dyn Error>> {
    // Create a Swarm to manage peers and events.
    let mut swarm = {
        SwarmBuilder::new(transport, behaviour, local_peer_id)
            // We want the connection background tasks to be spawned
            // onto the tokio runtime.
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };

    // Reach out to another node if specified
    if let Some(to_dial) = dial {
        swarm.dial_addr(to_dial.clone())?;
        println!("Dialed {:?}", to_dial)
    }

    // Listen on all interfaces and whatever port the OS assigns
    match transport_type {
        TransportType::Tcp => {
            swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", listen_on).parse()?)?;
        }
        TransportType::Ws => {
            swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}/ws", listen_on + 1).parse()?)?;
        }
    }
    Ok(swarm)
}
