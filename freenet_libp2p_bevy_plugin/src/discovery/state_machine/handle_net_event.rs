use crate::discovery;
use crate::net_id;
use crate::p2p;
use discovery::state_machine::{Output, State};

pub fn handle_net_event(state: &mut State, event: p2p::NetEvent, now: discovery::EpochSecs) {
    match event {
        p2p::NetEvent::PeerConnected(peer) => {
            state.connected.insert(peer.clone());
            discovery::state_machine::send_hello(state, &peer);
        }
        p2p::NetEvent::PeerDisconnected(peer) => {
            state.connected.remove(&peer);
            mark_lost(state, &peer, now);
        }
        p2p::NetEvent::Exchange { from, data } => {
            discovery::state_machine::handle_hello(state, &from, &data, now);
        }
        _ => return,
    }
    discovery::state_machine::dial_candidates(state, now);
}

// needed helper: keeps a dropped member for the grace window, marked not connected
fn mark_lost(state: &mut State, peer: &net_id::PeerId, now: discovery::EpochSecs) {
    let Some(member) = state
        .room
        .as_mut()
        .and_then(|room| room.members.get_mut(peer))
    else {
        return;
    };
    member.status = discovery::LinkStatus::Known;
    member.presence.updated_at = now;
    state
        .outputs
        .push(Output::Event(discovery::Event::MembersChanged));
}

#[cfg(test)]
mod tests {
    use super::handle_net_event;
    use crate::discovery;
    use crate::net_id;
    use crate::p2p;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        handle_net_event(
            &mut state,
            p2p::NetEvent::PeerConnected(net_id::PeerId::from("a")),
            discovery::EpochSecs(1),
        );
        assert!(state.connected.contains(&net_id::PeerId::from("a")));
        assert!(matches!(
            std::mem::take(&mut state.outputs).as_slice(),
            [Output::Net(p2p::NetCommand::Exchange { peer_id, .. })] if peer_id.as_str() == "a"
        ));
        handle_net_event(
            &mut state,
            p2p::NetEvent::PeerDisconnected(net_id::PeerId::from("a")),
            discovery::EpochSecs(2),
        );
        assert!(state.connected.is_empty());
    }
}
