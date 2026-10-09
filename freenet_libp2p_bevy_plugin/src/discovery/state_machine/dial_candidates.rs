use crate::discovery;
use crate::net_id;
use crate::p2p;
use discovery::state_machine::{Output, State};

pub fn dial_candidates(state: &mut State, now: discovery::EpochSecs) {
    if state.room.is_none() {
        return;
    }
    let redial_secs = state.timing.redial_secs;
    let due: Vec<net_id::Peer> = state
        .candidates
        .iter()
        .filter(|(peer, _)| !state.connected.contains(*peer))
        .filter(|(peer, _)| {
            state
                .last_dial
                .get(*peer)
                .is_none_or(|last| now.saturating_sub(**last) >= redial_secs)
        })
        .map(|(peer, presence)| net_id::Peer {
            id: peer.clone(),
            addrs: presence.addrs.clone(),
        })
        .collect();
    for peer in due {
        tracing::debug!(target: "room_lobby", peer = %peer.id.as_str(), "discovery dial candidate");
        state.last_dial.insert(peer.id.clone(), now);
        state.outputs.push(Output::Net(p2p::NetCommand::Dial(peer)));
    }
}

#[cfg(test)]
mod tests {
    use super::dial_candidates;
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
        state.room = Some(discovery::Room {
            name: net_id::RoomName::from("r"),
            members: discovery::Members::new(),
        });
        let peers = vec![
            net_id::Peer {
                id: net_id::PeerId::from("a"),
                addrs: vec![net_id::PeerAddr::from("/ip4/1")],
            },
            net_id::Peer {
                id: net_id::PeerId::from("b"),
                addrs: vec![net_id::PeerAddr::from("/ip4/2")],
            },
        ];
        discovery::state_machine::add_candidates(&mut state, peers, discovery::EpochSecs(1));
        state.connected.insert(net_id::PeerId::from("b"));
        dial_candidates(&mut state, discovery::EpochSecs(10));
        dial_candidates(&mut state, discovery::EpochSecs(10));
        assert!(matches!(
            std::mem::take(&mut state.outputs).as_slice(),
            [Output::Net(p2p::NetCommand::Dial(peer))] if peer.id.as_str() == "a"
        ));
        dial_candidates(&mut state, discovery::EpochSecs(12));
        assert_eq!(std::mem::take(&mut state.outputs).len(), 1);
    }
}
