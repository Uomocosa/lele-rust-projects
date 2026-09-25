use super::epoch_secs::EpochSecs;
use super::room_record::RoomRecord;

#[must_use]
pub fn occupancy(record: &RoomRecord, now: EpochSecs, ttl_secs: u64) -> usize {
    record
        .members
        .values()
        .filter(|presence| now.saturating_sub(*presence.updated_at) <= ttl_secs)
        .count()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::occupancy;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut members = BTreeMap::new();
        members.insert(
            discovery::id::RemotePeerId("fresh".to_string()),
            discovery::id::Presence {
                addrs: Vec::new(),
                updated_at: discovery::id::EpochSecs(90),
            },
        );
        members.insert(
            discovery::id::RemotePeerId("stale".to_string()),
            discovery::id::Presence {
                addrs: Vec::new(),
                updated_at: discovery::id::EpochSecs(10),
            },
        );
        let record = discovery::id::RoomRecord {
            capacity: 8,
            members,
        };
        assert_eq!(occupancy(&record, discovery::id::EpochSecs(100), 30), 1);
        assert_eq!(occupancy(&record, discovery::id::EpochSecs(100), 100), 2);
    }
}
