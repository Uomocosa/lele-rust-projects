use crate::discovery;
use discovery::Event;
use discovery::id::EpochSecs;
use discovery::session::Session;

pub fn prune_members(
    session: &mut Session,
    now: EpochSecs,
    grace_secs: u64,
    candidate_ttl_secs: u64,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    session
        .candidates
        .retain(|_, presence| now.saturating_sub(*presence.updated_at) <= candidate_ttl_secs);
    let connected = &session.connected;
    let Some(room) = session.room.as_mut() else {
        return;
    };
    let before = room.members.len();
    room.members.retain(|peer, member| {
        connected.contains(peer) || now.saturating_sub(*member.presence.updated_at) <= grace_secs
    });
    if room.members.len() != before {
        let _ = events.send(Event::MembersChanged);
    }
}

#[cfg(test)]
mod tests {
    use super::prune_members;
    use crate::discovery;
    use discovery::id::{EpochSecs, Presence, RemotePeerId, RoomName};
    use discovery::room_peers::{DiscoveryStatus, Member, Members};

    #[test]
    fn test_usage() {
        let (events, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("g".to_string()),
                &discovery::id::GameToken("t".to_string()),
            ),
            RemotePeerId("me".to_string()),
            Vec::new(),
            8,
        );
        let mut members = Members::new();
        for peer in ["live", "gone"] {
            members.insert(
                RemotePeerId(peer.to_string()),
                Member {
                    presence: Presence {
                        addrs: Vec::new(),
                        updated_at: EpochSecs(0),
                    },
                    status: DiscoveryStatus::Known,
                },
            );
        }
        session.room = Some(discovery::session::Room {
            name: RoomName("r".to_string()),
            members,
        });
        session.connected.insert(RemotePeerId("live".to_string()));
        prune_members(&mut session, EpochSecs(100), 10, 120, &events);
        let left: Vec<_> = session
            .room
            .as_ref()
            .map(|room| room.members.keys().cloned().collect())
            .unwrap_or_default();
        assert_eq!(left, vec![RemotePeerId("live".to_string())]);
    }
}
