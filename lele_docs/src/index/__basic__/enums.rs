#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemKind {
    #[default]
    Fn,
    Struct,
    Enum,
    Const,
    Static,
    TypeAlias,
    Trait,
    Mod,
}

// no test_usage necessary
