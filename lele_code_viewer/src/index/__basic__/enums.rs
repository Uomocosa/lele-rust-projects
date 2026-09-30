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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeBlock {
    Function,
    Struct,
    Enum,
    Trait,
}

// no test_usage necessary
