use std::path::{Path, PathBuf};

use crate::methods;
use crate::parse_source_files;
use crate::walk_entries;
use crate::Error;
use crate::ModuleInfo;
use crate::Project;
use crate::ScannedDir;

pub fn discover(
    start_dir: Option<&Path>,
    scan_folders: Option<&[PathBuf]>,
) -> Result<Project, Error> {
    let cwd = std::env::current_dir()?;
    let base_path = start_dir.unwrap_or(&cwd);

    if let Some(folders) = scan_folders {
        return discover_folders(base_path, folders);
    }

    let root = methods::project::find_cargo_root(base_path)?;
    let src_dir = root.join("src");
    if !src_dir.exists() || !src_dir.is_dir() {
        return Err(Error::NoSrcDirectory(src_dir));
    }
    let entries = walk_entries::walk_entries(&src_dir, &src_dir)?;
    let module_info = ModuleInfo::build(&src_dir, &entries);
    let (parsed_files, mut parse_failures) =
        parse_source_files::parse_source_files(&src_dir, &entries);
    let examples = scan_examples(&root)?;
    parse_failures.extend(examples.parse_failures);
    Ok(Project {
        root,
        src_dir,
        entries,
        module_info,
        parsed_files,
        example_entries: examples.entries,
        example_parsed_files: examples.parsed_files,
        parse_failures,
        ..Project::default()
    })
}

// needed helper: examples/ is linted by default alongside src/ and methods/
fn scan_examples(root: &Path) -> Result<ScannedDir, Error> {
    let dir = root.join("examples");
    if !dir.is_dir() {
        return Ok(ScannedDir::default());
    }
    let entries = walk_entries::walk_entries(&dir, &dir)?;
    let (parsed_files, parse_failures) = parse_source_files::parse_source_files(&dir, &entries);
    Ok(ScannedDir {
        entries,
        parsed_files,
        parse_failures,
    })
}

// needed helper: aggregate scanning over explicitly-passed folders (relative to the invocation base)
fn discover_folders(base_path: &Path, folders: &[PathBuf]) -> Result<Project, Error> {
    let mut entries = Vec::new();
    for folder in folders {
        let rel_dir = folder.strip_prefix("/").unwrap_or(folder.as_path());
        let abs_dir = base_path.join(rel_dir);
        if !abs_dir.exists() || !abs_dir.is_dir() {
            return Err(Error::NoScanFolder(abs_dir));
        }
        entries.extend(walk_entries::walk_entries(&abs_dir, base_path)?);
    }
    let module_info = ModuleInfo::build(base_path, &entries);
    let (parsed_files, parse_failures) =
        parse_source_files::parse_source_files(base_path, &entries);
    let owned_base = base_path.to_path_buf();
    Ok(Project {
        root: owned_base.clone(),
        src_dir: owned_base,
        entries,
        module_info,
        parsed_files,
        parse_failures,
        ..Project::default()
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::discover;
    use crate::Error;

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
    fn test_usage_examples_are_parsed_by_default() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='x'\nversion='0.0.0'\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        write_file(root, "src/a.rs");
        write_file(root, "examples/demo.rs");
        let project = discover(Some(root), None).unwrap();
        assert!(project
            .example_parsed_files
            .contains_key(&PathBuf::from("demo.rs")));
    }

    #[test]
    fn test_discover_folders_is_aggregated() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path();
        write_file(root, "src/a.rs");
        write_file(root, "contract/src/lib.rs");
        let folders = vec![PathBuf::from("src"), PathBuf::from("contract")];
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
        let folders = vec![PathBuf::from("nope")];
        assert!(matches!(
            discover(Some(root), Some(&folders)),
            Err(Error::NoScanFolder(_))
        ));
    }
}

// no test_usage necessary
