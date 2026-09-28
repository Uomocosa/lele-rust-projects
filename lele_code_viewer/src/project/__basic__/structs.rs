use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::index;

#[derive(Debug, Clone)]
pub struct ProjectRef {
    pub id: String,
    pub name: String,
    pub root: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub ignore: Vec<String>,
}

#[derive(Default)]
pub struct Registry {
    pub roots: Vec<PathBuf>,
    pub cache: RwLock<HashMap<String, Arc<index::SymbolIndex>>>,
    pub watch: bool,
    pub live: Mutex<LiveState>,
}

#[derive(Default)]
pub struct LiveState {
    pub watcher: Option<notify::RecommendedWatcher>,
    pub dir_refs: HashMap<PathBuf, usize>,
    pub projects: HashMap<String, LiveProject>,
    pub next_version: u64,
}

pub struct LiveProject {
    pub root: PathBuf,
    pub base_version: u64,
    pub version: u64,
    pub last_access: Instant,
    pub dirs: Vec<PathBuf>,
    pub changes: HashMap<PathBuf, FileChange>,
    pub notify: tokio::sync::watch::Sender<u64>,
}

pub struct FileChange {
    pub version: u64,
    pub at: Instant,
    pub lines: Vec<usize>,
    pub text: Option<String>,
}

pub struct ChangeFeed {
    pub version: u64,
    pub reset: bool,
    pub paths: Vec<String>,
}

// no test_usage necessary
