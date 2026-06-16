use bevy::prelude::Message;
use libp2p::gossipsub::TopicHash;
use libp2p::{Multiaddr, PeerId};

#[derive(Clone, Debug, Message)]
pub enum NetworkEvent {
    PeerDiscovered(PeerId, Multiaddr),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    Message(PeerId, TopicHash, Vec<u8>),
    NewListenAddr(Multiaddr),
}
