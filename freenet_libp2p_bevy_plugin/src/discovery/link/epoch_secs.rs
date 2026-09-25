#[must_use]
pub fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::epoch_secs;

    #[test]
    fn test_usage() {
        assert!(epoch_secs() > 1_700_000_000);
    }
}
