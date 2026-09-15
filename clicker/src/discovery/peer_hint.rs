use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerHint {
    pub peer_id: String,
    pub addrs: Vec<String>,
    pub updated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::PeerHint;

    #[test]
    fn test_usage() {
        let hint = PeerHint {
            peer_id: "peer".to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            updated_at: 7,
        };
        let bytes = bincode::serialize(&hint).unwrap_or_default();
        let decoded: PeerHint = bincode::deserialize(&bytes).unwrap_or(PeerHint {
            peer_id: String::new(),
            addrs: Vec::new(),
            updated_at: 0,
        });
        assert_eq!(decoded, hint);
    }
}
