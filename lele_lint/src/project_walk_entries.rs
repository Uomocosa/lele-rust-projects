use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;

use crate::entry;
use crate::entry_kind;
use crate::error;

pub(crate) fn walk_entries(src_dir: &Path) -> Result<Vec<entry::Entry>, error::Error> {
    let mut entries = Vec::new();
    for result in WalkDir::new(src_dir).min_depth(1) {
        let entry = result?;
        let absolute_path = entry.path().to_path_buf();
        let relative_path = absolute_path
            .strip_prefix(src_dir)
            .expect("entry under src_dir")
            .to_path_buf();
        if entry.file_type().is_dir() {
            entries.push(entry::Entry {
                relative_path,
                absolute_path,
                kind: entry_kind::EntryKind::Directory,
            });
        } else if entry.path().extension() == Some(OsStr::new("rs")) {
            entries.push(entry::Entry {
                relative_path,
                absolute_path,
                kind: entry_kind::EntryKind::File,
            });
        }
    }
    Ok(entries)
}

// no test_usage necessary
