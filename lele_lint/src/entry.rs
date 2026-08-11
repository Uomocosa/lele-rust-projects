use crate::entry_kind;

pub struct Entry {
    pub relative_path: std::path::PathBuf,
    pub absolute_path: std::path::PathBuf,
    pub kind: entry_kind::EntryKind,
}
