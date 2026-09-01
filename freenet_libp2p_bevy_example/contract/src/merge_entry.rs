use crate::error;
use crate::peer_entry;
use crate::verify_entry_signature;

pub fn merge_entry(
    existing: Option<&peer_entry::PeerEntry>,
    key: &[u8; 32],
    incoming: &peer_entry::PeerEntry,
) -> Result<Option<peer_entry::PeerEntry>, error::Error> {
    let Some(current) = existing else {
        verify_entry_signature::verify_entry_signature(key, incoming)?;
        return Ok(Some(incoming.clone()));
    };
    if incoming.seq < current.seq {
        return Err(error::Error::Rewind);
    }
    verify_entry_signature::verify_entry_signature(key, incoming)?;
    if incoming.seq == current.seq {
        return Ok(None);
    }
    Ok(Some(incoming.clone()))
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};

    use crate::{entry_bytes, error, peer_entry};

    use super::merge_entry;

    fn signed(secret: u8, seq: u64) -> ([u8; 32], peer_entry::PeerEntry) {
        let sk = SigningKey::from_bytes(&[secret; 32]);
        let vk = sk.verifying_key();
        let mut entry = peer_entry::PeerEntry {
            peer_id: format!("peer-{secret}"),
            addrs: Vec::new(),
            seq,
            signature: Vec::new(),
        };
        let sig = sk.sign(&entry_bytes::entry_signed_bytes(&entry));
        entry.signature = sig.to_bytes().to_vec();
        (vk.to_bytes(), entry)
    }

    #[test]
    fn test_usage() {
        let (key, entry) = signed(1, 1);

        assert!(merge_entry(None, &key, &entry).unwrap().is_some());

        let (_, advanced) = signed(1, 2);
        assert!(
            merge_entry(Some(&entry), &key, &advanced)
                .unwrap()
                .is_some()
        );

        let same = signed(1, 1).1;
        assert_eq!(merge_entry(Some(&entry), &key, &same).unwrap(), None);

        let (_, rewind) = signed(1, 0);
        assert_eq!(
            merge_entry(Some(&entry), &key, &rewind),
            Err(error::Error::Rewind)
        );

        let (other_key, _) = signed(2, 3);
        let (_, wrong_signer) = signed(9, 3);
        assert_eq!(
            merge_entry(Some(&entry), &other_key, &wrong_signer),
            Err(error::Error::SignatureInvalid)
        );
    }
}
