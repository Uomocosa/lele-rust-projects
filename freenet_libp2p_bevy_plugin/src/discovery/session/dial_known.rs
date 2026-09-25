use crate::discovery;
use crate::p2p;
use discovery::link::NetLink;
use discovery::room_peers::DiscoveryStatus;
use discovery::session::Session;

pub fn dial_known(session: &Session, link: &NetLink) {
    let Some(room) = &session.room else {
        return;
    };
    for (peer, member) in &room.members {
        if member.status != DiscoveryStatus::Connected && !member.presence.addrs.is_empty() {
            link.tx
                .send(p2p::NetCommand::Dial {
                    peer_id: (**peer).clone(),
                    addrs: member.presence.addrs.clone(),
                })
                .ok();
        }
    }
}

// no test_usage necessary
