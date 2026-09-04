use ed25519_dalek::SigningKey;

#[must_use]
pub fn signing_key_from_seed(seed: &[u8; 32]) -> SigningKey {
    SigningKey::from_bytes(seed)
}

#[cfg(test)]
mod tests {
    use super::signing_key_from_seed;

    #[test]
    fn test_usage() {
        let seed = [1u8; 32];
        let _ = signing_key_from_seed(&seed);
    }
}
