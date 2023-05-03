use color_eyre::eyre::Result;
use gossipnet::network::{Network, NetworkBuilder, Sha256Topic};

pub(crate) use gossipnet::network::Event;

const BLOCKS_TOPIC: &str = "blocks";

pub(crate) struct GossipNetwork(pub(crate) Network);

impl GossipNetwork {
    pub(crate) fn new(bootnodes: Vec<String>) -> Result<Self> {
        let mut network = NetworkBuilder::new().bootnodes(bootnodes).build()?;
        network.subscribe(&Sha256Topic::new(BLOCKS_TOPIC));
        Ok(Self(network))
    }
}
