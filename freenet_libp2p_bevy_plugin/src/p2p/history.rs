use libp2p::kad::RecordKey;
use serde::{Deserialize, Serialize};

pub const HISTORY_CHUNK: usize = 512;

pub fn history_key(lobby: &str, chunk: u64) -> RecordKey {
    RecordKey::new(&format!("blackboard/history/{lobby}/{chunk:08}"))
}

pub fn encode_chunk<T: Serialize>(stamps: &[T]) -> Vec<u8> {
    bincode::serialize(stamps).unwrap_or_default()
}

pub fn decode_chunk<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Vec<T> {
    bincode::deserialize(bytes).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{decode_chunk, encode_chunk, history_key};

    #[test]
    fn test_usage() {
        let key = history_key("lobby-a", 0);
        assert!(!key.as_ref().is_empty());
        let stamps = vec![1u32, 2, 3];
        let enc = encode_chunk(&stamps);
        let dec: Vec<u32> = decode_chunk(&enc);
        assert_eq!(stamps, dec);
    }
}
