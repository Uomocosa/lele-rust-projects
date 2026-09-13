use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerEntry {
    pub peer_id: String,
    pub addrs: Vec<String>,
    pub updated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::PeerEntry;

    #[test]
    fn test_usage() {
        let entry = PeerEntry {
            peer_id: "peer".to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            updated_at: 7,
        };
        let bytes = bincode::serialize(&entry).unwrap_or_default();
        let decoded: PeerEntry = bincode::deserialize(&bytes).unwrap_or(PeerEntry {
            peer_id: String::new(),
            addrs: Vec::new(),
            updated_at: 0,
        });
        assert_eq!(decoded, entry);
    }
}
