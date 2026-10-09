use crate::discovery;
use discovery::state_machine::{Output, State};

pub fn prune_members(state: &mut State, now: discovery::EpochSecs) {
    let ttl_secs = state.timing.presence_ttl_secs;
    let grace_secs = state.timing.member_grace_secs;
    state
        .candidates
        .retain(|_, presence| now.saturating_sub(*presence.updated_at) <= ttl_secs);
    let connected = &state.connected;
    let Some(room) = state.room.as_mut() else {
        return;
    };
    let before = room.members.len();
    room.members.retain(|peer, member| {
        connected.contains(peer) || now.saturating_sub(*member.presence.updated_at) <= grace_secs
    });
    if room.members.len() != before {
        state
            .outputs
            .push(Output::Event(discovery::Event::MembersChanged));
    }
}

#[cfg(test)]
mod tests {
    use super::prune_members;
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
        let mut members = discovery::Members::new();
        for peer in ["live", "gone"] {
            members.insert(
                net_id::PeerId(peer.to_string()),
                discovery::Member {
                    presence: discovery::Presence {
                        addrs: Vec::new(),
                        updated_at: discovery::EpochSecs(0),
                    },
                    status: discovery::LinkStatus::Known,
                },
            );
        }
        state.room = Some(discovery::Room {
            name: net_id::RoomName::from("r"),
            members,
        });
        state.connected.insert(net_id::PeerId::from("live"));
        prune_members(&mut state, discovery::EpochSecs(100));
        let left: Vec<_> = state
            .room
            .as_ref()
            .map(|room| room.members.keys().cloned().collect())
            .unwrap_or_default();
        assert_eq!(left, vec![net_id::PeerId::from("live")]);
    }
}
