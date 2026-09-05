use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub lobby: String,
    pub chunk: u64,
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::Chunk;

    #[test]
    fn test_usage() {
        let c = Chunk {
            lobby: "a".to_string(),
            chunk: 0,
            data: vec![1, 2],
        };
        assert_eq!(c.chunk, 0);
    }
}
