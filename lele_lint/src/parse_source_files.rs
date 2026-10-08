use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::Entry;
use crate::EntryKind;
use crate::ParseFailure;

pub(crate) fn parse_source_files(
    _src_dir: &Path,
    entries: &[Entry],
) -> (HashMap<PathBuf, syn::File>, Vec<ParseFailure>) {
    let mut map = HashMap::new();
    let mut failures = Vec::new();
    for entry in entries {
        if entry.kind != EntryKind::File {
            continue;
        }
        match std::fs::read_to_string(&entry.absolute_path) {
            Ok(content) => match syn::parse_file(&content) {
                Ok(file) => {
                    map.insert(entry.relative_path.clone(), file);
                }
                Err(e) => failures.push(ParseFailure {
                    path: entry.absolute_path.clone(),
                    cause: e.to_string(),
                }),
            },
            Err(e) => failures.push(ParseFailure {
                path: entry.absolute_path.clone(),
                cause: e.to_string(),
            }),
        }
    }
    (map, failures)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::path::PathBuf;

    use super::parse_source_files;
    use crate::Entry;
    use crate::EntryKind;

    fn file_entry(dir: &Path, name: &str) -> Entry {
        Entry {
            relative_path: PathBuf::from(name),
            absolute_path: dir.join(name),
            kind: EntryKind::File,
        }
    }

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "pub fn f() {}\n").unwrap();
        let entries = vec![file_entry(dir.path(), "a.rs")];
        let (parsed, failures) = parse_source_files(dir.path(), &entries);
        assert_eq!(parsed.len(), 1);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_usage_reports_unparseable_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "pub fn f() {}\n").unwrap();
        std::fs::write(dir.path().join("b.rs"), "pub fn f( {\n").unwrap();
        let entries = vec![
            file_entry(dir.path(), "a.rs"),
            file_entry(dir.path(), "b.rs"),
        ];
        let (parsed, failures) = parse_source_files(dir.path(), &entries);
        assert_eq!(parsed.len(), 1);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].path.ends_with("b.rs"));
    }
}
