use super::entry::Entry;

#[must_use]
pub fn merge_directory_entry(existing: Option<Entry>, incoming: Entry) -> Entry {
    match existing {
        Some(current) if current.updated_at >= incoming.updated_at => current,
        _ => incoming,
    }
}

#[cfg(test)]
mod tests {
    use super::merge_directory_entry;
    use crate::discovery;

    fn entry(updated_at: u64) -> discovery::directory::Entry {
        discovery::directory::Entry {
            params: vec![1],
            peer_id: "peer".to_string(),
            addrs: Vec::new(),
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(merge_directory_entry(None, entry(5)), entry(5));
        assert_eq!(merge_directory_entry(Some(entry(5)), entry(9)), entry(9));
    }
}
