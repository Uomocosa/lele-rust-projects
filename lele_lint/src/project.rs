// lele_lint: allow E001
// no test_usage necessary
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::Error;
use crate::module_info::{self, ModuleInfoMap};

pub struct Project {
    pub root: PathBuf,
    pub src_dir: PathBuf,
    pub entries: Vec<Entry>,
    pub module_info: ModuleInfoMap,
    pub parsed_files: HashMap<PathBuf, syn::File>,
}

pub struct Entry {
    pub relative_path: PathBuf,
    pub absolute_path: PathBuf,
    pub kind: EntryKind,
}

#[derive(PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

impl Project {
    pub fn get_parsed(&self, rel_path: &Path) -> Option<&syn::File> {
        self.parsed_files.get(rel_path)
    }
}

pub fn discover(start_dir: Option<&Path>) -> Result<Project, Error> {
    let cwd = std::env::current_dir()?;
    let search_from = start_dir.unwrap_or(&cwd);
    let root = find_cargo_root(search_from)?;
    let src_dir = root.join("src");

    if !src_dir.exists() || !src_dir.is_dir() {
        return Err(Error::NoSrcDirectory(src_dir.display().to_string()));
    }

    let entries = walk_entries(&src_dir)?;
    let module_info = module_info::build(&src_dir, &entries);
    let parsed_files = parse_source_files(&src_dir, &entries);

    Ok(Project {
        root,
        src_dir,
        entries,
        module_info,
        parsed_files,
    })
}

pub fn find_cargo_root(start: &Path) -> Result<PathBuf, Error> {
    let mut current = start.to_path_buf();
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(current);
        }
        if !current.pop() {
            return Err(Error::NoCargoRoot(start.display().to_string()));
        }
    }
}

fn walk_entries(src_dir: &Path) -> Result<Vec<Entry>, Error> {
    let mut entries = Vec::new();

    for result in WalkDir::new(src_dir).min_depth(1) {
        let entry = result?;
        let absolute_path = entry.path().to_path_buf();
        let relative_path = absolute_path
            .strip_prefix(src_dir)
            .expect("entry under src_dir")
            .to_path_buf();

        if entry.file_type().is_dir() {
            entries.push(Entry {
                relative_path,
                absolute_path,
                kind: EntryKind::Directory,
            });
        } else if entry.path().extension() == Some(OsStr::new("rs")) {
            entries.push(Entry {
                relative_path,
                absolute_path,
                kind: EntryKind::File,
            });
        }
    }

    Ok(entries)
}

fn parse_source_files(_src_dir: &Path, entries: &[Entry]) -> HashMap<PathBuf, syn::File> {
    let mut map = HashMap::new();
    for entry in entries {
        if entry.kind != EntryKind::File {
            continue;
        }
        let content = match std::fs::read_to_string(&entry.absolute_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if let Ok(file) = syn::parse_file(&content) {
            map.insert(entry.relative_path.clone(), file);
        }
    }
    map
}
