use crate::deliver::basic::constants::HASH_CHARS;

#[must_use]
pub fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes)
        .to_hex()
        .chars()
        .take(HASH_CHARS)
        .collect()
}

// no test_usage necessary
