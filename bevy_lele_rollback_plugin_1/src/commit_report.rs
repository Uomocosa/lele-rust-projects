pub struct CommitReport {
    pub tick: u64,
    pub diverged: bool,
    pub authoritative_hash: u64,
}

#[cfg(test)]
mod tests {
    use super::CommitReport;

    #[test]
    fn test_usage() {
        let report = CommitReport {
            tick: 2,
            diverged: true,
            authoritative_hash: 42,
        };
        assert_eq!(report.tick, 2);
        assert!(report.diverged);
        assert_eq!(report.authoritative_hash, 42);
    }
}
