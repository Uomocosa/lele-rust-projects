use std::path::Path;
use std::path::PathBuf;

use lele_lint::EntryKind;
use lele_lint::Project;

#[must_use]
pub fn file_path(project: &Project, rel_path: &Path) -> PathBuf {
    project
        .entries
        .iter()
        .find(|entry| entry.relative_path == rel_path && entry.kind == EntryKind::File)
        .map(|entry| entry.absolute_path.clone())
        .unwrap_or_else(|| project.src_dir.join(rel_path))
}

#[cfg(test)]
mod tests {
    use super::file_path;
    use lele_lint::Project;
    use std::path::Path;

    #[test]
    fn test_usage() {
        let project = Project::default();
        assert_eq!(
            file_path(&project, Path::new("a.rs")),
            project.src_dir.join("a.rs")
        );
    }
}
