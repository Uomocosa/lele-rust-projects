use std::path::Path;

use super::project::Project;
use crate::error;
use crate::module_info;
use crate::project_find_cargo_root;
use crate::project_parse_source_files;
use crate::project_walk_entries;

pub fn discover(
    start_dir: Option<&Path>,
    scan_folders: Option<&[String]>,
) -> Result<Project, error::Error> {
    let cwd = std::env::current_dir()?;
    let base = start_dir.unwrap_or(&cwd);

    if let Some(folders) = scan_folders {
        return discover_folders(base, folders);
    }

    let root = project_find_cargo_root::find_cargo_root(base)?;
    let src_dir = root.join("src");
    if !src_dir.exists() || !src_dir.is_dir() {
        return Err(error::Error::NoSrcDirectory(src_dir.display().to_string()));
    }
    let entries = project_walk_entries::walk_entries(&src_dir, &src_dir)?;
    let module_info = module_info::ModuleInfo::build(&src_dir, &entries);
    let parsed_files = project_parse_source_files::parse_source_files(&src_dir, &entries);
    Ok(Project {
        root,
        src_dir,
        entries,
        module_info,
        parsed_files,
    })
}

// needed helper: aggregate scanning over explicitly-passed folders (relative to the invocation base)
fn discover_folders(base: &Path, folders: &[String]) -> Result<Project, error::Error> {
    let mut entries = Vec::new();
    for folder in folders {
        let rel = folder.trim_start_matches('/');
        let abs = base.join(rel);
        if !abs.exists() || !abs.is_dir() {
            return Err(error::Error::NoScanFolder(abs.display().to_string()));
        }
        entries.extend(project_walk_entries::walk_entries(&abs, base)?);
    }
    let module_info = module_info::ModuleInfo::build(base, &entries);
    let parsed_files = project_parse_source_files::parse_source_files(base, &entries);
    let owned_base = base.to_path_buf();
    Ok(Project {
        root: owned_base.clone(),
        src_dir: owned_base,
        entries,
        module_info,
        parsed_files,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::discover;

    fn write_file(base: &Path, p: &str) {
        let path = base.join(p);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "pub fn f() {}\n").unwrap();
    }

    #[test]
    fn test_usage() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='x'\nversion='0.0.0'\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        write_file(root, "src/a.rs");
        let project = discover(Some(root), None).unwrap();
        assert!(project.parsed_files.contains_key(&PathBuf::from("a.rs")));
    }

    #[test]
    fn test_discover_folders_is_aggregated() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path();
        write_file(root, "src/a.rs");
        write_file(root, "contract/src/lib.rs");
        let folders = vec!["src".to_string(), "contract".to_string()];
        let project = discover(Some(root), Some(&folders)).unwrap();
        assert!(project
            .parsed_files
            .contains_key(&PathBuf::from("src/a.rs")));
        assert!(project
            .parsed_files
            .contains_key(&PathBuf::from("contract/src/lib.rs")));
        assert_eq!(project.src_dir, root);
    }

    #[test]
    fn test_discover_folders_missing_errors() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path();
        let folders = vec!["nope".to_string()];
        assert!(matches!(
            discover(Some(root), Some(&folders)),
            Err(crate::error::Error::NoScanFolder(_))
        ));
    }
}
