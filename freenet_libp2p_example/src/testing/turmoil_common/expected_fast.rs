#[must_use]
pub const fn expected_fast(acc_secs: u64) -> u64 {
    acc_secs.saturating_mul(8)
}

#[cfg(test)]
mod tests {
    use super::expected_fast;

    #[test]
    fn test_usage() {
        assert_eq!(expected_fast(20), 160);
    }
}
