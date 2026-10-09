use serde::Deserialize;

#[derive(PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Requirement {
    Honest,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Src,
    Methods,
    Examples,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub enum Category {
    #[display("Layout: files and folders")]
    Layout,
    #[display("Delegates: methods and `methods/`")]
    Delegates,
    #[display("Imports and re-exports")]
    Imports,
    #[display("Types")]
    Types,
    #[display("Tests")]
    Tests,
    #[display("Style")]
    Style,
    #[display("Crate config")]
    Config,
}
