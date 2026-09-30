use std::collections::BTreeSet;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::index::basic;

#[derive(Debug, Clone)]
pub struct IndexItem {
    pub id: String,
    pub name: String,
    pub module: String,
    pub kind: basic::enums::ItemKind,
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub name_line: usize,
    pub name_col_start: usize,
    pub name_col_end: usize,
    pub signature: String,
    pub doc: Option<String>,
    pub delegates_to: Option<String>,
    pub is_test: bool,
    pub external: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileDeps {
    pub internal: Vec<String>,
    pub external: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub module: String,
    pub external: Vec<String>,
    pub layer: usize,
}

#[derive(Debug, Clone)]
pub struct ModuleEdge {
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, Clone)]
pub struct ItemNode {
    pub id: String,
    pub name: String,
    pub kind: basic::enums::CodeBlock,
    pub external: Vec<String>,
    pub layer: usize,
}

#[derive(Debug, Clone)]
pub struct ItemEdge {
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, Default, Clone)]
pub struct ItemGraph {
    pub nodes: Vec<ItemNode>,
    pub edges: Vec<ItemEdge>,
    pub externals: Vec<String>,
    pub groups: Vec<basic::newtypes::ItemGroup>,
}

#[derive(Debug, Default, Clone)]
pub struct ModuleGraph {
    pub nodes: Vec<ModuleNode>,
    pub edges: Vec<ModuleEdge>,
    pub externals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Occurrence {
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub target: Option<String>,
    pub is_self: bool,
}

#[derive(Debug, Default, Clone)]
pub struct ExternalAliases {
    pub local_to_crate: HashMap<String, String>,
    pub glob_crates: BTreeSet<String>,
}

#[derive(Debug, Default, Clone)]
pub struct SymbolIndex {
    pub crate_name: String,
    pub root: PathBuf,
    pub items: Vec<IndexItem>,
    pub by_id: HashMap<String, usize>,
    pub by_name: HashMap<String, Vec<usize>>,
    pub by_module_name: HashMap<(String, String), Vec<usize>>,
    pub items_by_file: HashMap<PathBuf, Vec<usize>>,
    pub files: HashMap<PathBuf, String>,
    pub occurrences: HashMap<PathBuf, Vec<Occurrence>>,
    pub callers: HashMap<String, Vec<String>>,
    pub callees: HashMap<String, Vec<String>>,
    pub module_graph: ModuleGraph,
    pub item_graph: ItemGraph,
    pub markdown_files: Vec<PathBuf>,
    pub rust_files: Vec<PathBuf>,
    pub extra_files: Vec<PathBuf>,
}

// no test_usage necessary
