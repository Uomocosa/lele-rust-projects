use crate::input_log_entry;

pub fn log_signed_bytes(entry: &input_log_entry::InputLogEntry) -> Vec<u8> {
    bincode::serialize(&(entry.seq, entry.inputs.as_slice())).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::{hashed_input, input_log_entry};

    use super::log_signed_bytes;

    #[test]
    fn test_usage() {
        let entry = input_log_entry::InputLogEntry {
            seq: 3,
            inputs: vec![hashed_input::HashedInput { tick: 1, hash: 2 }],
            signature: vec![9; 64],
        };
        let bytes = log_signed_bytes(&entry);
        assert!(!bytes.is_empty());
        let mut other = entry.clone();
        other
            .inputs
            .push(hashed_input::HashedInput { tick: 2, hash: 3 });
        assert_ne!(log_signed_bytes(&entry), log_signed_bytes(&other));
    }
}
