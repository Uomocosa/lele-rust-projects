use crate::error;
use crate::params;
use crate::roster_state;
use crate::validate_entry;
use crate::verify_entry_signature;

pub fn validate_roster(
    roster: &roster_state::RosterState,
    params: &params::Params,
) -> Result<(), error::Error> {
    if roster.len() as u64 > params.max_members as u64 {
        return Err(error::Error::TooManyMembers);
    }
    for (key, entry) in roster {
        validate_entry::validate_entry(entry)?;
        verify_entry_signature::verify_entry_signature(key, entry)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};

    use crate::{entry_bytes, error, params, peer_entry, roster_state};

    use super::validate_roster;

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
        let p = params::Params {
            namespace: [7; 32],
            max_members: 1,
        };
        let (key_a, entry_a) = signed(1, 1);
        let mut roster = roster_state::RosterState::new();
        roster.insert(key_a, entry_a);
        assert!(validate_roster(&roster, &p).is_ok());

        let (key_b, entry_b) = signed(2, 1);
        roster.insert(key_b, entry_b);
        assert_eq!(
            validate_roster(&roster, &p),
            Err(error::Error::TooManyMembers)
        );

        let mut bad = roster_state::RosterState::new();
        let (key_c, mut entry_c) = signed(3, 1);
        entry_c.signature = vec![0; 64];
        bad.insert(key_c, entry_c);
        assert_eq!(
            validate_roster(&bad, &p),
            Err(error::Error::SignatureInvalid)
        );
    }
}
