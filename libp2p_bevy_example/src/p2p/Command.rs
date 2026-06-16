use libp2p::Multiaddr;

use crate::p2p::Message;

pub enum Command {
    Publish(libp2p::gossipsub::IdentTopic, Message),
    Dial(Multiaddr),
    GetPeers(tokio::sync::mpsc::Sender<Vec<libp2p::PeerId>>),
    SetEnableManualDial(bool),
}
