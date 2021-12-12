mod error;
use crate::behaviour::P2PBehaviour;
use async_std::sync::Mutex;
use error::*;
use futures::{channel::mpsc, StreamExt};
use libp2p::{
    core::{connection::ListenerId, muxing::StreamMuxerBox, transport::Boxed, upgrade},
    floodsub::{self, Floodsub, Topic},
    identity,
    mdns::Mdns,
    mplex, noise,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    Multiaddr, PeerId, Swarm, Transport,
};
use std::{collections::HashMap, error::Error};
use tokio::io::{self, AsyncBufReadExt};

pub struct P2PConfigBuilder {
    host: String,
    port: u16,
    private_key: identity::Keypair,
    peer_id: PeerId,
    peers: Vec<Multiaddr>,
}
impl Default for P2PConfigBuilder {
    fn default() -> Self {
        let (key, peer_id) = generate_identity();
        Self {
            host: "0.0.0.0".into(),
            port: 8500,
            private_key: key,
            peer_id,
            peers: Vec::new(),
        }
    }
}

impl P2PConfigBuilder {
    pub fn set_host(mut self, host: String) -> Self {
        self.host = host;
        self
    }
    pub fn set_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    pub fn set_private_key(mut self, private_key: identity::Keypair) -> Self {
        self.private_key = private_key;
        self
    }
    pub fn set_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id = peer_id;
        self
    }
    pub fn set_peers(mut self, peers: Vec<Multiaddr>) -> Self {
        self.peers = peers;
        self
    }
    pub fn build(mut self) -> P2PConfig {
        P2PConfig::from_builder(self)
    }
}

#[derive(Clone)]
pub struct P2PConfig {
    host: String,
    port: u16,
    private_key: identity::Keypair,
    peer_id: PeerId,
    peers: Vec<Multiaddr>,
}

impl Default for P2PConfig {
    fn default() -> Self {
        let (key, peer_id) = generate_identity();
        Self {
            host: "0.0.0.0".into(),
            port: 8500,
            private_key: key,
            peer_id,
            peers: Vec::new(),
        }
    }
}

impl P2PConfig {
    pub fn from_builder(builder: P2PConfigBuilder) -> Self {
        let P2PConfigBuilder {
            host,
            port,
            private_key,
            peer_id,
            peers,
        } = builder;
        Self {
            host,
            port,
            private_key,
            peer_id,
            peers,
        }
    }
    pub fn add_peer(&mut self, peer: Multiaddr) {
        self.peers.push(peer);
    }
}

type ClientMessage = String;
struct PeerMessage;

struct EventLoop {
    swarm: Swarm<P2PBehaviour>,
    message_receiver: mpsc::Receiver<ClientMessage>,
    topics: HashMap<String, Topic>,
}
impl EventLoop {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                    event = self.swarm.select_next_some() => match event{
                        SwarmEvent::NewListenAddr{address,..} =>println!("Listening on: {:?}", address),
                        _ => {}
                    },
            message = self.message_receiver.next() => {
                match message{
                            Some(c) => self.handle_incoming_message(c).await,
                            None=>  return,
                        }
                }
            }
        }
    }
    //async fn handle_event(&mut self, event: SwarmEvent<>) {
    //    todo!()
    //}
    async fn handle_incoming_message(&mut self, message: ClientMessage) {
        self.swarm.behaviour_mut().floodsub.publish(
            self.topics.get("Communication").unwrap().clone(),
            message.as_bytes(),
        )
    }

    pub async fn dial(&mut self, addr: Multiaddr) -> P2PResult<()> {
        Ok(self.swarm.dial_addr(addr)?)
    }

    pub async fn start_listen(&mut self, host: String, port: u16) -> P2PResult<ListenerId> {
        let listen_addr: Multiaddr = format!("/ip4/{}/tcp/{}", host, port).parse()?;
        println!("trying to listen on: {}", listen_addr);
        Ok(self.swarm.listen_on(listen_addr)?)
    }
}

pub struct P2PServer {
    host: String,
    port: u16,
    private_key: identity::Keypair,
    peer_id: PeerId,
    peers: Vec<Multiaddr>,
    runnig: bool,
    lock: Mutex<()>,
    message_sender: Option<mpsc::Sender<ClientMessage>>,
}

impl P2PServer {
    pub fn new(config: P2PConfig) -> P2PResult<Self> {
        let P2PConfig {
            host,
            port,
            private_key,
            peer_id,
            peers,
        } = config;

        Ok(Self {
            host,
            port,
            private_key,
            peers,
            peer_id,
            runnig: false,
            lock: Mutex::new(()),
            message_sender: None,
        })
    }
    pub async fn start(&mut self) -> P2PResult<()> {
        self.lock.lock().await;
        if self.runnig {
            eprintln!("Server already running");
            Err(Box::new(P2PError::ServerRunning))
        } else {
            self.runnig = true;
            let (message_sender, message_receiver) = mpsc::channel(0);
            self.message_sender = Some(message_sender);
            let mdns = futures::executor::block_on(generate_mdns())?;
            let topic = generate_floodsub_topic("P2PNodeCommunicationTopic");
            //let transport = config_transport(&self.private_key);
            let transport = libp2p::development_transport(self.private_key.clone()).await?;
            let behaviour = futures::executor::block_on(floodsub_behaviour(
                self.peer_id.clone(),
                mdns,
                topic.clone(),
            ));
            let swarm = futures::executor::block_on(swarm_config(
                transport,
                behaviour,
                self.peer_id.clone(),
            ));
            let topics = HashMap::from([("Communication".to_string(), topic)]);
            let mut event_loop = EventLoop {
                swarm,
                topics,
                message_receiver,
            };
            event_loop
                .start_listen(self.host.clone(), self.port)
                .await?;
            for p in &self.peers {
                event_loop.dial(p.clone()).await?;
            }
            tokio::spawn(event_loop.run());
            let mut message_sender = self.message_sender.as_ref().unwrap().clone();
            tokio::spawn(async move {
                let mut stdin = io::BufReader::new(io::stdin()).lines();
                loop {
                    tokio::select! {
                    line = stdin.next_line() => {
                    let line = format!(r#"{}"#,line.unwrap().unwrap());
                    match message_sender.try_send(line.clone()){
                        Ok(_) => {println!("Sent data: {}", line)},
                        Err(_) => {
                            println!("Retrying to send data");
                            message_sender.try_send(line);}
                    }
                    },
                    }
                }
            });
            Ok(())
        }
    }
}

pub fn generate_identity() -> (identity::Keypair, PeerId) {
    let local_key: identity::Keypair = identity::Keypair::generate_ed25519();
    let local_peer_id: PeerId = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);
    (local_key, local_peer_id)
}

pub async fn generate_mdns() -> Result<Mdns, Box<dyn Error>> {
    Ok(Mdns::new(Default::default()).await?)
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

pub fn generate_floodsub_topic(name: &str) -> floodsub::Topic {
    floodsub::Topic::new(name)
}

pub async fn floodsub_behaviour(
    local_peer_id: PeerId,
    mdns: Mdns,
    topic: floodsub::Topic,
) -> P2PBehaviour {
    let mut behaviour = P2PBehaviour {
        floodsub: Floodsub::new(local_peer_id.clone()),
        mdns,
    };
    behaviour.floodsub.subscribe(topic.clone());
    behaviour
}

pub async fn swarm_config(
    transport: Boxed<(PeerId, StreamMuxerBox)>,
    behaviour: P2PBehaviour,
    local_peer_id: PeerId,
) -> Swarm<P2PBehaviour> {
    SwarmBuilder::new(transport, behaviour, local_peer_id)
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build()
}
