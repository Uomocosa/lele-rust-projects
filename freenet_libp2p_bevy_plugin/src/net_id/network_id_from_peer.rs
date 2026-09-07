use super::network_id::NetworkId;

pub fn from_peer(peer: &str) -> NetworkId {
    let mut value = 0u64;
    for byte in blake3::hash(peer.as_bytes()).as_bytes().iter().take(8) {
        value = value.wrapping_shl(8).wrapping_add(u64::from(*byte));
    }
    NetworkId(value)
}

#[cfg(test)]
mod tests {
    use super::from_peer;

    #[test]
    fn test_usage() {
        let first = from_peer("peer-a");
        assert_eq!(first, from_peer("peer-a"));
        assert_ne!(first, from_peer("peer-b"));
    }
}
