use super::snapshot::Snapshot;

#[must_use]
pub fn encode_snapshot(snapshot: &Snapshot) -> Vec<u8> {
    bincode::serialize(snapshot).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::encode_snapshot;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let snapshot = clicker::Snapshot::default();
        let bytes = encode_snapshot(&snapshot);
        assert_eq!(clicker::decode_snapshot(&bytes), Some(snapshot));
    }
}
