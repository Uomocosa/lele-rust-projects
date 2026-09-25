use derive_more::Deref;
use serde::{Deserialize, Serialize};

use super::super::id::remote_peer_id::RemotePeerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
pub struct MeshMessage(pub Vec<(RemotePeerId, Vec<String>)>);

#[cfg(test)]
mod tests {
    use super::MeshMessage;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let message = MeshMessage(vec![(
            discovery::id::RemotePeerId("peer".to_string()),
            vec!["/ip4/127.0.0.1/tcp/1".to_string()],
        )]);
        assert_eq!(message.len(), 1);
        let bytes = bincode::serialize(&message).unwrap_or_default();
        let decoded: MeshMessage = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, message);
    }
}
