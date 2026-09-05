use derive_more::Deref;

#[derive(Debug, Clone, Deref)]
pub struct ExcludedPath(pub String);

#[cfg(test)]
mod tests {
    use super::ExcludedPath;

    #[test]
    fn test_usage() {
        let excluded = ExcludedPath("target".to_string());
        assert_eq!(excluded.as_str(), "target");
    }
}
