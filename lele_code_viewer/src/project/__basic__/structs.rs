use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use crate::index;

#[derive(Debug, Clone)]
pub struct ProjectRef {
    pub id: String,
    pub name: String,
    pub root: PathBuf,
}

#[derive(Default)]
pub struct Registry {
    pub roots: Vec<PathBuf>,
    pub cache: RwLock<HashMap<String, Arc<index::SymbolIndex>>>,
}

// no test_usage necessary
