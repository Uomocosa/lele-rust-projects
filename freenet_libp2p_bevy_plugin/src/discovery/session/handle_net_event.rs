use crate::discovery;
use crate::p2p;
use discovery::session::{Output, Session};

pub fn handle_net_event(session: &mut Session, event: p2p::TapEvent, now: discovery::EpochSecs) {
    match event {
        p2p::TapEvent::PeerConnected(peer) => {
            let peer = discovery::PeerId(peer);
            session.connected.insert(peer.clone());
            discovery::session::send_hello(session, &peer);
        }
        p2p::TapEvent::PeerDisconnected(peer) => {
            let peer = discovery::PeerId(peer);
            session.connected.remove(&peer);
            mark_lost(session, &peer, now);
        }
        p2p::TapEvent::Exchange { from, data } => {
            discovery::session::handle_hello(session, &discovery::PeerId(from), &data, now);
        }
        _ => return,
    }
    discovery::session::dial_candidates(session, now);
}

// needed helper: keeps a dropped member for the grace window, marked not connected
fn mark_lost(session: &mut Session, peer: &discovery::PeerId, now: discovery::EpochSecs) {
    let Some(member) = session
        .room
        .as_mut()
        .and_then(|room| room.members.get_mut(peer))
    else {
        return;
    };
    member.status = discovery::LinkStatus::Known;
    member.presence.updated_at = now;
    session
        .outputs
        .push(Output::Event(discovery::Event::MembersChanged));
}

#[cfg(test)]
mod tests {
    use super::handle_net_event;
    use crate::discovery;
    use crate::p2p;
    use discovery::session::{Output, Session};

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        handle_net_event(
            &mut session,
            p2p::TapEvent::PeerConnected("a".to_string()),
            discovery::EpochSecs(1),
        );
        assert!(
            session
                .connected
                .contains(&discovery::PeerId("a".to_string()))
        );
        assert!(matches!(
            std::mem::take(&mut session.outputs).as_slice(),
            [Output::Net(p2p::NetCommand::Exchange { peer_id, .. })] if peer_id == "a"
        ));
        handle_net_event(
            &mut session,
            p2p::TapEvent::PeerDisconnected("a".to_string()),
            discovery::EpochSecs(2),
        );
        assert!(session.connected.is_empty());
    }
}
