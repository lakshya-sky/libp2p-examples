use libp2p::Multiaddr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(short, long, default_value = "3000")]
    pub port: u16,
    pub dial: Option<Multiaddr>,
}
