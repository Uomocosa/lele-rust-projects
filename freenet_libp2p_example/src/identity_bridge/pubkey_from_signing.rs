use ed25519_dalek::{SigningKey, VerifyingKey};

use crate::identity_bridge;

#[must_use]
pub fn pubkey_from_signing(sk: &SigningKey) -> identity_bridge::Pubkey {
    *VerifyingKey::from(sk).as_bytes()
}

#[cfg(test)]
mod tests {
    use super::pubkey_from_signing;
    use crate::identity_bridge::signing_key_from_seed::signing_key_from_seed;

    #[test]
    fn test_usage() {
        let sk = signing_key_from_seed(&[2u8; 32]);
        let _ = pubkey_from_signing(&sk);
    }
}
