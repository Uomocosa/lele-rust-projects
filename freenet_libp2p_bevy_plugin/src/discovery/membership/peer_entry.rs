use serde::{Deserialize, Serialize};

use super::super::params::epoch_secs::EpochSecs;
use super::super::params::remote_peer_id::RemotePeerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerEntry {
    pub peer_id: RemotePeerId,
    pub addrs: Vec<String>,
    pub updated_at: EpochSecs,
}

#[cfg(test)]
mod tests {
    use super::PeerEntry;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let entry = PeerEntry {
            peer_id: discovery::params::RemotePeerId("peer".to_string()),
            addrs: Vec::new(),
            updated_at: discovery::params::EpochSecs(1),
        };
        let bytes = bincode::serialize(&entry).unwrap_or_default();
        let decoded: PeerEntry = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, entry);
    }
}
