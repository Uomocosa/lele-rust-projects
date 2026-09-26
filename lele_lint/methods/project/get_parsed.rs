use std::path::Path;

use crate::Project;

pub fn get_parsed<'a>(project: &'a Project, rel_path: &Path) -> Option<&'a syn::File> {
    project.parsed_files.get(rel_path)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::get_parsed;
    use crate::Project;

    #[test]
    fn test_usage() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("a.rs"),
            syn::parse_str("pub fn f() {}\n").unwrap(),
        );
        assert!(get_parsed(&project, Path::new("a.rs")).is_some());
        assert!(get_parsed(&project, Path::new("missing.rs")).is_none());
    }
}
