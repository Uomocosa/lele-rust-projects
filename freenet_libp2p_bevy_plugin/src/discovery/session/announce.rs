use std::collections::BTreeMap;

use crate::discovery;
use crate::p2p;
use discovery::link::NetLink;
use discovery::room_peers::{MeshMessage, peer_topic};
use discovery::session::Session;

pub fn announce(session: &Session, link: &NetLink) {
    let Some(room) = &session.room else {
        return;
    };
    let mut peers = BTreeMap::new();
    peers.insert(session.me.clone(), session.addrs.clone());
    for (peer, member) in &room.members {
        peers.insert(peer.clone(), member.presence.addrs.clone());
    }
    let message = MeshMessage(peers.into_iter().collect());
    let data = bincode::serialize(&message).unwrap_or_default();
    link.tx
        .send(p2p::NetCommand::Publish {
            topic: peer_topic(&session.id, &room.name),
            data,
        })
        .ok();
}

// no test_usage necessary
