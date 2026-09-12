pub struct Report {
    pub hooks_per_crate: Vec<(String, usize)>,
    pub config_path: String,
    pub pointer_updated: bool,
}

#[cfg(test)]
mod tests {
    use super::Report;

    #[test]
    fn test_usage() {
        let report = Report {
            hooks_per_crate: vec![("clicker".to_owned(), 5)],
            config_path: "/root/.pre-commit-config.yaml".to_owned(),
            pointer_updated: true,
        };
        assert_eq!(report.hooks_per_crate.len(), 1);
        assert!(report.pointer_updated);
    }
}
