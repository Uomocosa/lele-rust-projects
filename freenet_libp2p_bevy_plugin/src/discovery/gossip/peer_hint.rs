use serde::{Deserialize, Serialize};

use super::super::params::epoch_secs::EpochSecs;
use super::super::params::remote_peer_id::RemotePeerId;
use super::super::params::room_name::RoomName;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerHint {
    pub peer_id: RemotePeerId,
    pub addrs: Vec<String>,
    pub rooms: Vec<RoomName>,
    pub updated_at: EpochSecs,
}

#[cfg(test)]
mod tests {
    use super::PeerHint;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let hint = PeerHint {
            peer_id: discovery::params::RemotePeerId("peer".to_string()),
            addrs: Vec::new(),
            rooms: vec![discovery::params::RoomName("room-1".to_string())],
            updated_at: discovery::params::EpochSecs(7),
        };
        assert_eq!(hint.rooms.len(), 1);
    }
}
