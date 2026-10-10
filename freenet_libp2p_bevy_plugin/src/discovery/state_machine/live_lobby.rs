use crate::discovery;

#[must_use]
pub fn live_lobby(
    lobby: discovery::Lobby,
    now: discovery::EpochSecs,
    ttl_secs: u64,
) -> discovery::Lobby {
    lobby
        .into_iter()
        .filter_map(|(room, mut record)| {
            record
                .members
                .retain(|_, presence| now.saturating_sub(*presence.updated_at) <= ttl_secs);
            (!record.members.is_empty()).then_some((room, record))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use std::collections::BTreeMap;

    use super::live_lobby;
    use crate::discovery;

    fn presence(updated_at: u64) -> discovery::Presence {
        discovery::Presence {
            addrs: Vec::new(),
            updated_at: discovery::EpochSecs(updated_at),
        }
    }

    #[test]
    fn test_usage() {
        let mut lobby = discovery::Lobby::new();
        lobby.insert(
            net_id::RoomName::from("live"),
            discovery::RoomEntry {
                capacity: 8,
                members: BTreeMap::from([
                    (net_id::PeerId::from("fresh"), presence(90)),
                    (net_id::PeerId::from("stale"), presence(10)),
                ]),
            },
        );
        lobby.insert(
            net_id::RoomName::from("dead"),
            discovery::RoomEntry {
                capacity: 8,
                members: BTreeMap::from([(net_id::PeerId::from("gone"), presence(1))]),
            },
        );
        let live = live_lobby(lobby, discovery::EpochSecs(100), 30);
        let rooms: Vec<_> = live.keys().map(|room| room.as_str()).collect();
        assert_eq!(rooms, vec!["live"]);
        let members = live
            .get(&net_id::RoomName::from("live"))
            .map_or(0, |record| record.members.len());
        assert_eq!(members, 1);
    }
}
