use super::snapshot::Snapshot;

#[must_use]
pub fn decode_snapshot(bytes: &[u8]) -> Option<Snapshot> {
    bincode::deserialize(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::decode_snapshot;
    use crate::clicker;

    #[test]
    fn test_usage() {
        assert!(decode_snapshot(b"garbage").is_none());
        let snapshot = clicker::Snapshot::default();
        let bytes = clicker::encode_snapshot(&snapshot);
        assert_eq!(decode_snapshot(&bytes), Some(snapshot));
    }
}
