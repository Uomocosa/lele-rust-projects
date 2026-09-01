use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::entry_bytes;
use crate::error;
use crate::peer_entry;

pub fn verify_entry_signature(
    key: &[u8; 32],
    entry: &peer_entry::PeerEntry,
) -> Result<(), error::Error> {
    let payload = entry_bytes::entry_signed_bytes(entry);
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

    use crate::{entry_bytes, error, peer_entry};

    use super::verify_entry_signature;

    fn signed(secret: u8, seq: u64) -> ([u8; 32], peer_entry::PeerEntry) {
        let sk = SigningKey::from_bytes(&[secret; 32]);
        let vk = sk.verifying_key();
        let mut entry = peer_entry::PeerEntry {
            peer_id: "peer".to_string(),
            addrs: vec![format!("/ip4/127.0.0.1/tcp/{seq}")],
            seq,
            signature: Vec::new(),
        };
        let sig = sk.sign(&entry_bytes::entry_signed_bytes(&entry));
        entry.signature = sig.to_bytes().to_vec();
        (vk.to_bytes(), entry)
    }

    #[test]
    fn test_usage() {
        let (key, entry) = signed(7, 1);
        assert!(verify_entry_signature(&key, &entry).is_ok());

        let mut other = entry.clone();
        other.signature = vec![0; 64];
        assert_eq!(
            verify_entry_signature(&key, &other),
            Err(error::Error::SignatureInvalid)
        );

        let (other_key, _) = signed(8, 1);
        assert_eq!(
            verify_entry_signature(&other_key, &entry),
            Err(error::Error::SignatureInvalid)
        );
    }
}
