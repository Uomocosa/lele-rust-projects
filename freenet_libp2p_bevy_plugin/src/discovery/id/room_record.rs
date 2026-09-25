use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::presence::Presence;
use super::remote_peer_id::RemotePeerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomRecord {
    pub capacity: u16,
    pub members: BTreeMap<RemotePeerId, Presence>,
}

#[cfg(test)]
mod tests {
    use super::RoomRecord;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut members = std::collections::BTreeMap::new();
        members.insert(
            discovery::id::RemotePeerId("peer".to_string()),
            discovery::id::Presence {
                addrs: Vec::new(),
                updated_at: discovery::id::EpochSecs(1),
            },
        );
        let record = RoomRecord {
            capacity: 8,
            members,
        };
        let bytes = bincode::serialize(&record).unwrap_or_default();
        let decoded: RoomRecord = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, record);
    }
}
