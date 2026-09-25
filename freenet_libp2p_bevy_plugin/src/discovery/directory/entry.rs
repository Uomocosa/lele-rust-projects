use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub params: Vec<u8>,
    pub peer_id: String,
    pub addrs: Vec<String>,
    pub updated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::Entry;

    #[test]
    fn test_usage() {
        let entry = Entry {
            params: vec![1, 2],
            peer_id: "peer".to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            updated_at: 7,
        };
        let bytes = bincode::serialize(&entry).unwrap_or_default();
        let decoded: Entry = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, entry);
    }
}
