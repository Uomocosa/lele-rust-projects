// no test_usage necessary
// needed helper: filesystem and source parsing utilities
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::project_discover;
use super::project_find_cargo_root;
use super::project_get_parsed;
use crate::entry::Entry;
use crate::entry_kind::EntryKind;
use crate::error::Error;
use crate::module_info::ModuleInfoMap;

pub struct Project {
    pub root: PathBuf,
    pub src_dir: PathBuf,
    pub entries: Vec<Entry>,
    pub module_info: ModuleInfoMap,
    pub parsed_files: HashMap<PathBuf, syn::File>,
}

#[rustfmt::skip]
impl Project {
    pub fn get_parsed(&self, rel_path: &Path) -> Option<&syn::File> {
        project_get_parsed::get_parsed(self, rel_path)
    }
}

#[rustfmt::skip]
impl Project {
    pub fn discover(start_dir: Option<&Path>) -> Result<Self, Error> {
        project_discover::discover(start_dir)
    }
    pub fn find_cargo_root(start: &Path) -> Result<PathBuf, Error> {
        project_find_cargo_root::find_cargo_root(start)
    }
}

pub(crate) fn walk_entries(src_dir: &Path) -> Result<Vec<Entry>, Error> {
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

pub(crate) fn parse_source_files(
    _src_dir: &Path,
    entries: &[Entry],
) -> HashMap<PathBuf, syn::File> {
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
