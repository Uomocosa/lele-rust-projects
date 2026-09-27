pub fn is_hidden_dir(name: &str) -> bool {
    matches!(
        name,
        "target" | ".git" | ".devenv" | "node_modules" | "__OLD__"
    )
}

#[cfg(test)]
mod tests {
    use super::is_hidden_dir;

    #[test]
    fn test_usage() {
        assert!(is_hidden_dir("target"));
        assert!(is_hidden_dir(".git"));
        assert!(!is_hidden_dir("src"));
    }
}
