use crate::peer_entry;

pub fn entry_signed_bytes(entry: &peer_entry::PeerEntry) -> Vec<u8> {
    bincode::serialize(&(entry.peer_id.as_str(), entry.addrs.as_slice(), entry.seq))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::{entry_bytes, peer_entry};

    #[test]
    fn test_usage() {
        let entry = peer_entry::PeerEntry {
            peer_id: "peer".to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/1".to_string()],
            seq: 3,
            signature: vec![9; 64],
        };
        let bytes = entry_bytes::entry_signed_bytes(&entry);
        assert!(!bytes.is_empty());
        let mut other = entry.clone();
        other.seq = 4;
        assert_ne!(
            entry_bytes::entry_signed_bytes(&entry),
            entry_bytes::entry_signed_bytes(&other)
        );
    }
}
