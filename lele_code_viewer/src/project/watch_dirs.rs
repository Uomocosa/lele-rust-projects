use std::path::Path;
use std::path::PathBuf;

use crate::project;

pub fn watch_dirs(root: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| {
            entry.file_type().is_dir()
                && (entry.depth() == 0
                    || !entry
                        .file_name()
                        .to_str()
                        .is_some_and(project::is_hidden_dir))
        })
        .flatten()
        .map(walkdir::DirEntry::into_path)
        .take(project::MAX_WATCH_DIRS)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::watch_dirs;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/sub")).unwrap();
        fs::create_dir_all(dir.path().join("target/debug")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "").unwrap();
        let dirs = watch_dirs(dir.path());
        assert_eq!(dirs.len(), 3);
        assert!(dirs.iter().all(|d| !d.to_string_lossy().contains("target")));
    }
}
