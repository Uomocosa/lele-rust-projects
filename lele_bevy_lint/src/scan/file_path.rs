use std::path::Path;
use std::path::PathBuf;

use lele_lint::Origin;
use lele_lint::Project;

#[must_use]
pub fn file_path(project: &Project, origin: Origin, rel_path: &Path) -> PathBuf {
    project.absolute_path(origin, rel_path)
}

#[cfg(test)]
mod tests {
    use super::file_path;
    use lele_lint::Origin;
    use lele_lint::Project;
    use std::path::Path;

    #[test]
    fn test_usage() {
        let project = Project::default();
        assert_eq!(
            file_path(&project, Origin::Src, Path::new("a.rs")),
            project.src_dir.join("a.rs")
        );
    }
}
