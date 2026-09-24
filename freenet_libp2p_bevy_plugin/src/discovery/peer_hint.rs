use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerHint {
    pub peer_id: String,
    pub addrs: Vec<String>,
    pub rooms: Vec<String>,
    pub updated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::PeerHint;

    #[test]
    fn test_usage() {
        let hint = PeerHint {
            peer_id: "peer".to_string(),
            addrs: Vec::new(),
            rooms: vec!["room-1".to_string()],
            updated_at: 7,
        };
        assert_eq!(hint.rooms.len(), 1);
    }
}
