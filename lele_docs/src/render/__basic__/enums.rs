#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    Index,
    File,
    Item,
    Md,
    Search,
    Asset,
}

// no test_usage necessary
