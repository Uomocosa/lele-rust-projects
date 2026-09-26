#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}
