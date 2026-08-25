use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashedInput {
    pub tick: u64,
    pub hash: u64,
}

#[cfg(test)]
mod tests {
    use super::HashedInput;

    #[test]
    fn test_usage() {
        let input = HashedInput { tick: 1, hash: 99 };
        let encoded = bincode::serialize(&input);
        let decoded = encoded.ok().and_then(|e| bincode::deserialize(&e).ok());
        assert_eq!(decoded, Some(input));
    }
}
