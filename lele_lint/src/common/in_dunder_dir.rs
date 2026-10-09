use std::path::Path;

use crate::DunderPath;

pub(crate) fn in_dunder_dir(rel_path: &Path, dunder_paths: &[DunderPath]) -> bool {
    rel_path.components().any(|component| {
        dunder_paths.iter().any(|dunder| {
            dunder.is_dunder_dir() && dunder.file_name() == Some(component.as_os_str())
        })
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::in_dunder_dir;
    use crate::common;

    #[test]
    fn test_usage() {
        let dunder_paths = common::default_dunder_paths();
        assert!(in_dunder_dir(
            Path::new("player/__basic__/enums.rs"),
            &dunder_paths
        ));
        assert!(!in_dunder_dir(Path::new("player/player.rs"), &dunder_paths));
    }
}
