use std::path::Path;

use lele_lint::Project;

use crate::scan;

#[must_use]
pub fn dir_spawns_ui(project: &Project, rel_path: &Path) -> bool {
    let dir = rel_path.parent();
    project
        .parsed_files
        .iter()
        .any(|(other, file)| other.parent() == dir && !scan::prod_visuals(file).is_empty())
}

#[cfg(test)]
mod tests {
    use super::dir_spawns_ui;
    use lele_lint::Project;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_usage() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("ui/spawn.rs"),
            syn::parse_str("pub fn spawn_ui(s: &mut S) { s.spawn((Node,)); }").unwrap(),
        );
        assert!(dir_spawns_ui(&project, Path::new("ui/plugin.rs")));
        assert!(!dir_spawns_ui(&project, Path::new("p2p/plugin.rs")));
        assert!(!dir_spawns_ui(&project, Path::new("plugin.rs")));
    }
}
