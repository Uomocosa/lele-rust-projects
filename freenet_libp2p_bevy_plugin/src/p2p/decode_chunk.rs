#[must_use]
pub fn decode_chunk<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Vec<T> {
    bincode::deserialize(bytes).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::decode_chunk;
    use crate::p2p::encode_chunk;

    #[test]
    fn test_usage() {
        let stamps = vec![1u32, 2, 3];
        let enc = encode_chunk(&stamps);
        let dec: Vec<u32> = decode_chunk(&enc);
        assert_eq!(stamps, dec);
    }
}
