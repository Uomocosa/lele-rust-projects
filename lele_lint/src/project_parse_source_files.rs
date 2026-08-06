use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::entry::Entry;
use crate::entry_kind::EntryKind;

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

// no test_usage necessary
