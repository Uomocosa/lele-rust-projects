use crate::discovery;

#[must_use]
pub fn now_epoch() -> discovery::EpochSecs {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    discovery::EpochSecs(secs)
}

#[cfg(test)]
mod tests {
    use super::now_epoch;

    #[test]
    fn test_usage() {
        assert!(*now_epoch() > 1_700_000_000);
    }
}
