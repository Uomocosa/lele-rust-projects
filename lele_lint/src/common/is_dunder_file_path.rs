use std::path::Path;

use crate::DunderPath;

pub(crate) fn is_dunder_file_path(rel_path: &Path, dunder_paths: &[DunderPath]) -> bool {
    dunder_paths
        .iter()
        .any(|dunder| dunder.is_dunder_file() && dunder.file_stem() == rel_path.file_stem())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::is_dunder_file_path;
    use crate::DunderPath;

    #[test]
    fn test_usage() {
        let dunder_paths = [DunderPath::try_from(PathBuf::from("__prelude__.rs")).unwrap()];
        assert!(is_dunder_file_path(
            Path::new("player/__prelude__.rs"),
            &dunder_paths
        ));
        assert!(!is_dunder_file_path(
            Path::new("player/player.rs"),
            &dunder_paths
        ));
    }
}
