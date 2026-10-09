use crate::discovery;
use discovery::state_machine::State;

pub fn tick(state: &mut State, now: discovery::EpochSecs) {
    discovery::state_machine::dial_candidates(state, now);
    let hello_due = state
        .last_hello
        .is_none_or(|last| now.saturating_sub(*last) >= state.timing.hello_secs);
    if hello_due && state.room.is_some() {
        discovery::state_machine::broadcast_hello(state);
        state.last_hello = Some(now);
    }
    discovery::state_machine::prune_members(state, now);
}

#[cfg(test)]
mod tests {
    use super::tick;
    use crate::discovery;
    use crate::net_id;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        state.connected.insert(net_id::PeerId::from("a"));
        state.room = Some(discovery::Room {
            name: net_id::RoomName::from("r"),
            members: discovery::Members::new(),
        });
        tick(&mut state, discovery::EpochSecs(100));
        tick(&mut state, discovery::EpochSecs(101));
        assert_eq!(state.last_hello, Some(discovery::EpochSecs(100)));
        assert_eq!(std::mem::take(&mut state.outputs).len(), 1);
    }
}
