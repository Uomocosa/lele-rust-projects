use serde::Serialize;

pub fn encode_chunk<T: Serialize>(stamps: &[T]) -> Vec<u8> {
    bincode::serialize(stamps).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::encode_chunk;

    #[test]
    fn test_usage() {
        let stamps = vec![1u32, 2, 3];
        let enc = encode_chunk(&stamps);
        assert_ne!(enc.len(), 0);
    }
}
