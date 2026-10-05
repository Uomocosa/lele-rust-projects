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
