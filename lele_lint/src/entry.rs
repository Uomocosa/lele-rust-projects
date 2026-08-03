pub struct Entry {
    pub relative_path: std::path::PathBuf,
    pub absolute_path: std::path::PathBuf,
    pub kind: crate::entry_kind::EntryKind,
}
