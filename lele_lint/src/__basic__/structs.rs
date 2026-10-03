use serde::Deserialize;

use crate::basic;
use crate::basic::enums::Requirement;
use crate::Dunder;

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct LeleSection {
    #[serde(default)]
    pub lint: LeleTomlLintSections,
    #[serde(default)]
    pub vocabulary: Vec<VocabularyEntry>,
    #[serde(default)]
    pub boundary: Vec<BoundaryEntry>,
    #[serde(default, rename = "config")]
    pub enforce_config: Option<toml::Value>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct VocabularyEntry {
    pub name: String,
    pub meaning: String,
    pub banned: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BoundaryEntry {
    pub name: String,
    pub why: String,
    pub folders: Vec<String>,
    #[serde(default)]
    pub cannot_use: Vec<String>,
    #[serde(default)]
    pub require: Option<Requirement>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct LeleTomlLintSections {
    #[serde(default)]
    pub dunder_whitelist: Dunder,
    #[serde(default)]
    pub clippy_allow_whitelist: Vec<AllowWhitelistEntry>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AllowWhitelistEntry {
    pub allow: String,
    pub file: String,
    pub reason: String,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub file: std::path::PathBuf,
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub message: String,
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

#[derive(Debug, Clone, Copy)]
pub struct ExampleFile {
    pub path: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct RuleDoc {
    pub category: &'static str,
    pub summary: &'static str,
    pub why: &'static str,
    pub bad: &'static [ExampleFile],
    pub good: &'static [ExampleFile],
}
