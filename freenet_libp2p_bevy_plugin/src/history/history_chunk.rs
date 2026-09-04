use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryChunk {
    pub lobby: String,
    pub chunk: u64,
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::HistoryChunk;

    #[test]
    fn test_usage() {
        let c = HistoryChunk {
            lobby: "a".to_string(),
            chunk: 0,
            data: vec![1, 2],
        };
        assert_eq!(c.chunk, 0);
    }
}
