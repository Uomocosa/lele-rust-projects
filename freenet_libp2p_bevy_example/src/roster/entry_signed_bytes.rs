pub fn entry_signed_bytes(peer_id: &str, addrs: &[String], seq: u64) -> Vec<u8> {
    bincode::serialize(&(peer_id, addrs, seq)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::entry_signed_bytes;

    #[test]
    fn test_usage() {
        let a = entry_signed_bytes("peer", &["/ip4/127.0.0.1/tcp/1".to_string()], 3);
        assert_eq!(
            a,
            vec![
                4, 0, 0, 0, 0, 0, 0, 0, 112, 101, 101, 114, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0,
                0, 0, 0, 47, 105, 112, 52, 47, 49, 50, 55, 46, 48, 46, 48, 46, 49, 47, 116, 99,
                112, 47, 49, 3, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn stable_and_distinct_under_seq_change() {
        let a = entry_signed_bytes("peer", &["/a".to_string()], 1);
        let again = entry_signed_bytes("peer", &["/a".to_string()], 1);
        assert_eq!(a, again);

        let b = entry_signed_bytes("peer", &["/a".to_string()], 2);
        assert_ne!(a, b);
    }
}
