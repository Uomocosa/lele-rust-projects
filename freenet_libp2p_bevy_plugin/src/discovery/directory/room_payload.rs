use serde::{Deserialize, Serialize};

use super::super::params::contract_params::ContractParams;
use super::super::params::epoch_secs::EpochSecs;
use super::super::params::remote_peer_id::RemotePeerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomPayload {
    pub params: ContractParams,
    pub peer_id: RemotePeerId,
    pub addrs: Vec<String>,
    pub updated_at: EpochSecs,
}

#[cfg(test)]
mod tests {
    use super::RoomPayload;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let payload = RoomPayload {
            params: discovery::params::ContractParams(vec![1, 2]),
            peer_id: discovery::params::RemotePeerId("peer".to_string()),
            addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            updated_at: discovery::params::EpochSecs(7),
        };
        let bytes = bincode::serialize(&payload).unwrap_or_default();
        let decoded: RoomPayload = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, payload);
    }
}
