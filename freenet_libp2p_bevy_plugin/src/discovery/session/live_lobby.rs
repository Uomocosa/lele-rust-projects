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
            discovery::RoomName("live".to_string()),
            discovery::RoomRecord {
                capacity: 8,
                members: BTreeMap::from([
                    (discovery::PeerId("fresh".to_string()), presence(90)),
                    (discovery::PeerId("stale".to_string()), presence(10)),
                ]),
            },
        );
        lobby.insert(
            discovery::RoomName("dead".to_string()),
            discovery::RoomRecord {
                capacity: 8,
                members: BTreeMap::from([(discovery::PeerId("gone".to_string()), presence(1))]),
            },
        );
        let live = live_lobby(lobby, discovery::EpochSecs(100), 30);
        let rooms: Vec<_> = live.keys().map(|room| room.as_str()).collect();
        assert_eq!(rooms, vec!["live"]);
        let members = live
            .get(&discovery::RoomName("live".to_string()))
            .map_or(0, |record| record.members.len());
        assert_eq!(members, 1);
    }
}
