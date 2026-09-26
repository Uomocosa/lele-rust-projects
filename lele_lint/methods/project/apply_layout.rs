use crate::project_parse_source_files;
use crate::project_walk_entries;
use crate::Config;
use crate::Error;
use crate::Project;

pub fn apply_layout(project: &mut Project, config: &Config) -> Result<(), Error> {
    project.dunder = config.dunder();

    let methods_dir = project.root.join("methods");
    if !methods_dir.is_dir() {
        return Ok(());
    }

    let entries = project_walk_entries::walk_entries(&methods_dir, &methods_dir)?;
    let parsed_files = project_parse_source_files::parse_source_files(&methods_dir, &entries);
    project.methods_dir = Some(methods_dir);
    project.methods_entries = entries;
    project.methods_parsed_files = parsed_files;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply_layout;
    use crate::Config;
    use crate::Project;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let mut project = Project::default();
        project.root = dir.path().to_path_buf();
        apply_layout(&mut project, &Config::default()).unwrap();
        assert!(project.methods_dir.is_none());
        std::fs::create_dir_all(dir.path().join("methods")).unwrap();
        std::fs::write(
            dir.path().join("methods").join("probe.rs"),
            "pub fn probe() {}\n",
        )
        .unwrap();
        apply_layout(&mut project, &Config::default()).unwrap();
        assert!(project.methods_dir.is_some());
    }
}
