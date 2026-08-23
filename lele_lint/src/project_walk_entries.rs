use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;

use crate::entry;
use crate::entry_kind;
use crate::error;

pub(crate) fn walk_entries(
    walk_root: &Path,
    strip_base: &Path,
) -> Result<Vec<entry::Entry>, error::Error> {
    let mut entries = Vec::new();
    let walker = WalkDir::new(walk_root)
        .min_depth(1)
        .into_iter()
        .filter_entry(keep_entry);
    for result in walker {
        let entry = result?;
        let absolute_path = entry.path().to_path_buf();
        let relative_path = absolute_path
            .strip_prefix(strip_base)
            .map_err(|_| error::Error::Io(std::io::Error::other("entry not under strip base")))?
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

fn keep_entry(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }
    !is_skipped_name(entry.file_name().to_str())
}

fn is_skipped_name(name: Option<&str>) -> bool {
    matches!(name, Some("target" | ".git" | "node_modules"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::walk_entries;

    #[test]
    fn test_usage() {
        use crate::entry_kind::EntryKind;

        let base = tempfile::tempdir().unwrap();
        let root = base.path();
        fs::create_dir_all(root.join("src/nested")).unwrap();
        fs::create_dir_all(root.join("src/target")).unwrap();
        fs::create_dir_all(root.join("src/.git")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn x() {}\n").unwrap();
        fs::write(root.join("src/nested/a.rs"), "pub fn y() {}\n").unwrap();
        fs::write(root.join("src/target/b.rs"), "pub fn z() {}\n").unwrap();
        fs::write(root.join("src/.git/c.rs"), "pub fn w() {}\n").unwrap();

        let entries = walk_entries(&root.join("src"), root).unwrap();
        let rel: Vec<PathBuf> = entries
            .iter()
            .filter(|e| e.kind == EntryKind::File)
            .map(|e| e.relative_path.clone())
            .collect();
        assert!(rel.contains(&PathBuf::from("src/lib.rs")));
        assert!(rel.contains(&PathBuf::from("src/nested/a.rs")));
        assert!(!rel.iter().any(|p| p.to_string_lossy().contains("target")));
        assert!(!rel.iter().any(|p| p.to_string_lossy().contains(".git")));
    }
}
