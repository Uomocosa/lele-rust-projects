use crate::discovery;
use crate::net_id;
use discovery::state_machine::State;

pub fn broadcast_hello(state: &mut State) {
    let peers: Vec<net_id::PeerId> = state.connected.iter().cloned().collect();
    for peer in &peers {
        discovery::state_machine::send_hello(state, peer);
    }
}

#[cfg(test)]
mod tests {
    use super::broadcast_hello;
    use crate::discovery;
    use crate::net_id;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: vec![net_id::PeerAddr::from("/ip4/9")],
            },
            discovery::Timing::default(),
        );
        state.connected.insert(net_id::PeerId::from("a"));
        state.connected.insert(net_id::PeerId::from("b"));
        broadcast_hello(&mut state);
        assert_eq!(std::mem::take(&mut state.outputs).len(), 2);
    }
}
