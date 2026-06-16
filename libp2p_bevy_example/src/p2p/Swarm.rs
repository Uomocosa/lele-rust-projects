use libp2p::PeerId;
use tokio::sync::mpsc;

use crate::p2p::Command;
use crate::p2p::Config;
use crate::p2p::Error;
use crate::p2p::Message;
use crate::p2p::NetworkEvent;
use crate::p2p::SwarmMethod;

#[allow(dead_code)]
pub struct Swarm {
    pub local_peer_id: PeerId,
    pub(crate) command_sender: mpsc::Sender<Command>,
    pub(crate) config: Config,
}

#[rustfmt::skip]
impl Swarm {
    pub fn new(config: Config) -> Result<(Self, mpsc::Receiver<NetworkEvent>), Error> { SwarmMethod::new(config) }
    pub fn dial(&mut self, addr: libp2p::Multiaddr) { SwarmMethod::dial(self, addr) }
    pub fn get_connected_peers(&mut self) -> Vec<PeerId> { SwarmMethod::get_connected_peers(self) }
    pub fn publish(&mut self, topic: libp2p::gossipsub::IdentTopic, message: Message) { SwarmMethod::publish(self, topic, message) }
    pub fn set_enable_manual_dial(&mut self, enabled: bool) { SwarmMethod::set_enable_manual_dial(self, enabled) }
}
