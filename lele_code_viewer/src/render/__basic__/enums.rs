#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    Index,
    File,
    Item,
    Md,
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

// no test_usage necessary
