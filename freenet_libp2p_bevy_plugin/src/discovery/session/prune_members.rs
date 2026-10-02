use crate::discovery;
use discovery::session::{Output, Session};

pub fn prune_members(session: &mut Session, now: discovery::EpochSecs) {
    let ttl_secs = session.timing.presence_ttl_secs;
    let grace_secs = session.timing.member_grace_secs;
    session
        .candidates
        .retain(|_, presence| now.saturating_sub(*presence.updated_at) <= ttl_secs);
    let connected = &session.connected;
    let Some(room) = session.room.as_mut() else {
        return;
    };
    let before = room.members.len();
    room.members.retain(|peer, member| {
        connected.contains(peer) || now.saturating_sub(*member.presence.updated_at) <= grace_secs
    });
    if room.members.len() != before {
        session
            .outputs
            .push(Output::Event(discovery::Event::MembersChanged));
    }
}

#[cfg(test)]
mod tests {
    use super::prune_members;
    use crate::discovery;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        let mut members = discovery::Members::new();
        for peer in ["live", "gone"] {
            members.insert(
                discovery::PeerId(peer.to_string()),
                discovery::Member {
                    presence: discovery::Presence {
                        addrs: Vec::new(),
                        updated_at: discovery::EpochSecs(0),
                    },
                    status: discovery::LinkStatus::Known,
                },
            );
        }
        session.room = Some(discovery::Room {
            name: discovery::RoomName("r".to_string()),
            members,
        });
        session
            .connected
            .insert(discovery::PeerId("live".to_string()));
        prune_members(&mut session, discovery::EpochSecs(100));
        let left: Vec<_> = session
            .room
            .as_ref()
            .map(|room| room.members.keys().cloned().collect())
            .unwrap_or_default();
        assert_eq!(left, vec![discovery::PeerId("live".to_string())]);
    }
}
