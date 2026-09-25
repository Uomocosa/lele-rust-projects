#[must_use]
pub fn dialable(addrs: Vec<String>) -> Vec<String> {
    addrs
        .into_iter()
        .filter(|a| !a.contains("0.0.0.0"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::dialable;

    #[test]
    fn test_usage() {
        assert_eq!(
            dialable(vec!["/ip4/0.0.0.0/tcp/9000".to_string()]),
            Vec::<String>::new()
        );
        let addrs = dialable(vec!["/ip4/1.2.3.4/tcp/9000".to_string()]);
        assert_eq!(addrs.len(), 1);
    }
}
