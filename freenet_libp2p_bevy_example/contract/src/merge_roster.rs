use crate::error;
use crate::merge_entry;
use crate::roster_state;

pub fn merge_roster(
    base: &mut roster_state::RosterState,
    incoming: roster_state::RosterState,
) -> Result<(), error::Error> {
    for (key, entry) in incoming {
        let existing = base.get(&key).cloned();
        let Some(merged) = merge_entry::merge_entry(existing.as_ref(), &key, &entry)? else {
            continue;
        };
        base.insert(key, merged);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};

    use crate::{entry_bytes, peer_entry, roster_state};

    use super::merge_roster;

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
        let (key_a, entry_a) = signed(1, 1);
        let (key_b, entry_b) = signed(2, 1);

        let mut base = roster_state::RosterState::new();
        let mut incoming = roster_state::RosterState::new();
        incoming.insert(key_a, entry_a);
        incoming.insert(key_b, entry_b);
        merge_roster(&mut base, incoming).unwrap();

        assert_eq!(base.len(), 2);
        assert!(base.contains_key(&key_a));
        assert!(base.contains_key(&key_b));
    }
}
