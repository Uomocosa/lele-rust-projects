use std::collections::HashMap;

use serde::Deserialize;

use crate::basic;
use crate::Dunder;

#[derive(Deserialize, Debug, Default)]
pub struct LeleLintSection {
    #[serde(default)]
    pub checkers: HashMap<String, bool>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub dunder: Dunder,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub file: std::path::PathBuf,
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub message: String,
    pub severity: basic::enums::Severity,
}

pub struct Entry {
    pub relative_path: std::path::PathBuf,
    pub absolute_path: std::path::PathBuf,
    pub kind: basic::enums::EntryKind,
}

#[derive(Debug, Clone)]
pub struct ModDecl {
    pub name: String,
    pub is_public: bool,
    pub cfg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Reexport {
    pub segments: Vec<String>,
    pub is_glob: bool,
}
