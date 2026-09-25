use super::epoch_secs::EpochSecs;

#[must_use]
pub fn now_epoch() -> EpochSecs {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    EpochSecs(secs)
}

#[cfg(test)]
mod tests {
    use super::now_epoch;

    #[test]
    fn test_usage() {
        assert!(*now_epoch() > 1_700_000_000);
    }
}
