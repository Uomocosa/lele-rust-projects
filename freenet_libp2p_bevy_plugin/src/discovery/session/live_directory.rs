use crate::discovery;

#[must_use]
pub fn live_directory(
    directory: discovery::Directory,
    now: discovery::EpochSecs,
    ttl_secs: u64,
) -> discovery::Directory {
    directory
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

    use super::live_directory;
    use crate::discovery;

    fn presence(updated_at: u64) -> discovery::Presence {
        discovery::Presence {
            addrs: Vec::new(),
            updated_at: discovery::EpochSecs(updated_at),
        }
    }

    #[test]
    fn test_usage() {
        let mut directory = discovery::Directory::new();
        directory.insert(
            discovery::RoomName("live".to_string()),
            discovery::RoomRecord {
                capacity: 8,
                members: BTreeMap::from([
                    (discovery::PeerId("fresh".to_string()), presence(90)),
                    (discovery::PeerId("stale".to_string()), presence(10)),
                ]),
            },
        );
        directory.insert(
            discovery::RoomName("dead".to_string()),
            discovery::RoomRecord {
                capacity: 8,
                members: BTreeMap::from([(discovery::PeerId("gone".to_string()), presence(1))]),
            },
        );
        let live = live_directory(directory, discovery::EpochSecs(100), 30);
        let rooms: Vec<_> = live.keys().map(|room| room.as_str()).collect();
        assert_eq!(rooms, vec!["live"]);
        let members = live
            .get(&discovery::RoomName("live".to_string()))
            .map_or(0, |record| record.members.len());
        assert_eq!(members, 1);
    }
}
