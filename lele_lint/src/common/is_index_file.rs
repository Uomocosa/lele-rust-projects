pub(crate) fn is_index_file(file_name: &str) -> bool {
    matches!(file_name, "mod.rs" | "lib.rs" | "main.rs")
}

#[cfg(test)]
mod tests {
    use super::is_index_file;

    #[test]
    fn test_usage() {
        assert!(is_index_file("mod.rs"));
        assert!(is_index_file("lib.rs"));
        assert!(!is_index_file("greet.rs"));
    }
}
