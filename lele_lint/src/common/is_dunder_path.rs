use std::path::Path;

use crate::common;
use crate::DunderPath;

pub(crate) fn is_dunder_path(rel_path: &Path, dunder_paths: &[DunderPath]) -> bool {
    common::in_dunder_dir(rel_path, dunder_paths)
        || common::is_dunder_file_path(rel_path, dunder_paths)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::is_dunder_path;
    use crate::DunderPath;

    #[test]
    fn test_usage() {
        let dunder_paths = [
            DunderPath::try_from(PathBuf::from("__basic__")).unwrap(),
            DunderPath::try_from(PathBuf::from("__prelude__.rs")).unwrap(),
        ];
        assert!(is_dunder_path(
            Path::new("player/__basic__/enums.rs"),
            &dunder_paths
        ));
        assert!(is_dunder_path(
            Path::new("player/__prelude__.rs"),
            &dunder_paths
        ));
        assert!(!is_dunder_path(
            Path::new("player/player.rs"),
            &dunder_paths
        ));
    }
}
