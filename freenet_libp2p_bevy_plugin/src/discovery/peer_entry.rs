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
            addrs: Vec::new(),
            updated_at: 1,
        };
        let bytes = bincode::serialize(&entry).unwrap_or_default();
        let decoded: PeerEntry = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, entry);
    }
}
