use std::sync::Arc;

use crate::index;
use crate::source;

pub struct AppState {
    pub idx: Arc<index::SymbolIndex>,
    pub hl: Arc<source::Highlighter>,
}

// no test_usage necessary
