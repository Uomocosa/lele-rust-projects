use crate::constants;
use crate::error;
use crate::peer_entry;

pub fn validate_entry(entry: &peer_entry::PeerEntry) -> Result<(), error::Error> {
    if entry.addrs.len() > constants::MAX_ADDRS {
        return Err(error::Error::TooManyAddrs);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{constants, error, peer_entry};

    use super::validate_entry;

    #[test]
    fn test_usage() {
        let entry = peer_entry::PeerEntry {
            peer_id: "peer".to_string(),
            addrs: (0..constants::MAX_ADDRS)
                .map(|i| format!("/x/{i}"))
                .collect(),
            seq: 1,
            signature: Vec::new(),
        };
        assert!(validate_entry(&entry).is_ok());

        let mut many = entry.clone();
        many.addrs.push("/x/extra".to_string());
        assert_eq!(validate_entry(&many), Err(error::Error::TooManyAddrs));
    }
}
