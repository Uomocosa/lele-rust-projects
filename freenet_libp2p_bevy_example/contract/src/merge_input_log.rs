use crate::error;
use crate::input_log_entry;
use crate::verify_log_signature;

pub fn merge_input_log(
    existing: Option<&input_log_entry::InputLogEntry>,
    key: &[u8; 32],
    incoming: &input_log_entry::InputLogEntry,
) -> Result<Option<input_log_entry::InputLogEntry>, error::Error> {
    let Some(current) = existing else {
        verify_log_signature::verify_log_signature(key, incoming)?;
        return Ok(Some(incoming.clone()));
    };
    if incoming.seq < current.seq {
        return Err(error::Error::InputLogRewind);
    }
    verify_log_signature::verify_log_signature(key, incoming)?;
    if incoming.seq == current.seq {
        return Ok(None);
    }
    Ok(Some(incoming.clone()))
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};

    use crate::{error, hashed_input, input_log_entry, log_signed_bytes};

    use super::merge_input_log;

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
        let (key, entry) = signed(1, 1);
        assert!(merge_input_log(None, &key, &entry).unwrap().is_some());

        let (_, advanced) = signed(1, 2);
        assert!(
            merge_input_log(Some(&entry), &key, &advanced)
                .unwrap()
                .is_some()
        );

        let same = signed(1, 1).1;
        assert_eq!(merge_input_log(Some(&entry), &key, &same).unwrap(), None);

        let (_, rewind) = signed(1, 0);
        assert_eq!(
            merge_input_log(Some(&entry), &key, &rewind),
            Err(error::Error::InputLogRewind)
        );

        let (_, mut unsigned) = signed(1, 3);
        unsigned.signature = Vec::new();
        assert_eq!(
            merge_input_log(Some(&entry), &key, &unsigned),
            Err(error::Error::UnsignedInput)
        );
    }
}
