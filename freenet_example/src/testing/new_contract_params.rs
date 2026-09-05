#[must_use]
pub fn new_contract_params() -> String {
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use std::collections::BTreeSet;
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut m = BTreeSet::new();
    for tag in 0..=10u64 {
        let mut seed = [0u8; 32];
        seed[0..8].copy_from_slice(&tag.to_le_bytes());
        let sk = SigningKey::from_bytes(&seed);
        let vk = VerifyingKey::from(&sk);
        m.insert(*vk.as_bytes());
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let bytes = nanos.to_le_bytes();
    let mut seed = [0u8; 32];
    seed[0..8].copy_from_slice(&bytes[0..8]);
    seed[8..16].copy_from_slice(&bytes[8..16]);
    let sk = SigningKey::from_bytes(&seed);
    let vk = VerifyingKey::from(&sk);
    m.insert(*vk.as_bytes());
    let bytes = bincode::serialize(&m).unwrap_or_default();
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::new_contract_params;

    #[test]
    fn test_usage() {
        let a = new_contract_params();
        let b = new_contract_params();
        assert_ne!(a, "");
        assert_ne!(a, b);
    }
}
