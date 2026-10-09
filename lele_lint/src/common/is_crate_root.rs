use std::path::Path;

pub(crate) fn is_crate_root(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| matches!(n, "lib.rs" | "main.rs"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::is_crate_root;

    #[test]
    fn test_usage() {
        assert!(is_crate_root(Path::new("lib.rs")));
        assert!(is_crate_root(Path::new("main.rs")));
        assert!(is_crate_root(Path::new("src/main.rs")));
        assert!(!is_crate_root(Path::new("player/mod.rs")));
    }
}
