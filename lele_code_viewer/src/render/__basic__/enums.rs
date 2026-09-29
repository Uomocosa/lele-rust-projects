#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    Index,
    File,
    Md,
    Raw,
    Search,
    Asset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewKind {
    #[default]
    None,
    Files,
    Deps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Rust,
    Text,
    Image,
    Video,
    Audio,
    Other,
}

// no test_usage necessary
