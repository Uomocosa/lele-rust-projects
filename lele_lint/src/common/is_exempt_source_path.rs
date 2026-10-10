use std::path::Path;

use crate::common;
use crate::DunderPath;

pub(crate) fn is_exempt_source_path(rel_path: &Path, dunder_paths: &[DunderPath]) -> bool {
    let file_name = rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    matches!(file_name, "mod.rs" | "lib.rs" | "constants.rs")
        || common::in_dunder_dir(rel_path, dunder_paths)
        || rel_path
            .components()
            .any(|c| c.as_os_str().to_str() == Some("tests"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::is_exempt_source_path;

    #[test]
    fn test_usage() {
        assert!(is_exempt_source_path(Path::new("a/mod.rs"), &[]));
        assert!(is_exempt_source_path(Path::new("a/constants.rs"), &[]));
        assert!(!is_exempt_source_path(Path::new("a/foo.rs"), &[]));
    }
}
