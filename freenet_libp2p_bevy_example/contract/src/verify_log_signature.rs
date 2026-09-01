use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::error;
use crate::input_log_entry;
use crate::log_signed_bytes;

pub fn verify_log_signature(
    key: &[u8; 32],
    entry: &input_log_entry::InputLogEntry,
) -> Result<(), error::Error> {
    if entry.signature.is_empty() {
        return Err(error::Error::UnsignedInput);
    }
    let payload = log_signed_bytes::log_signed_bytes(entry);
    let Ok(vk) = VerifyingKey::from_bytes(key) else {
        return Err(error::Error::SignatureInvalid);
    };
    let Ok(signature) = Signature::from_slice(&entry.signature) else {
        return Err(error::Error::SignatureInvalid);
    };
    vk.verify(&payload, &signature)
        .map_err(|_| error::Error::SignatureInvalid)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};

    use crate::{hashed_input, input_log_entry, log_signed_bytes};

    use super::verify_log_signature;

    fn signed(secret: u8, seq: u64) -> ([u8; 32], input_log_entry::InputLogEntry) {
        let sk = SigningKey::from_bytes(&[secret; 32]);
        let vk = sk.verifying_key();
        let mut entry = input_log_entry::InputLogEntry {
            seq,
            inputs: vec![hashed_input::HashedInput {
                tick: seq,
                hash: seq,
            }],
            signature: Vec::new(),
        };
        let sig = sk.sign(&log_signed_bytes::log_signed_bytes(&entry));
        entry.signature = sig.to_bytes().to_vec();
        (vk.to_bytes(), entry)
    }

    #[test]
    fn test_usage() {
        let (key, entry) = signed(7, 1);
        assert!(verify_log_signature(&key, &entry).is_ok());

        let mut other = entry.clone();
        other.signature = vec![0; 64];
        assert_eq!(
            verify_log_signature(&key, &other),
            Err(crate::error::Error::SignatureInvalid)
        );

        let (other_key, _) = signed(8, 1);
        assert_eq!(
            verify_log_signature(&other_key, &entry),
            Err(crate::error::Error::SignatureInvalid)
        );
    }
}
