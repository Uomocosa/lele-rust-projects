use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TaxonomyCtx {
    pub declared_honest: HashSet<String>,
    pub declared_dishonest: HashSet<String>,
    pub honesty_depth: usize,
    pub entry_allowlist: Vec<String>,
    pub manifest_path: Option<PathBuf>,
}

impl Default for TaxonomyCtx {
    fn default() -> Self {
        Self {
            declared_honest: HashSet::new(),
            declared_dishonest: HashSet::new(),
            honesty_depth: 1,
            entry_allowlist: vec!["src/main.rs".to_string(), "examples/**".to_string()],
            manifest_path: None,
        }
    }
}

impl TaxonomyCtx {
    pub fn is_declared_honest(&self, path: &str) -> bool {
        self.declared_honest
            .iter()
            .any(|p| path == p || path.ends_with(p))
    }

    pub fn is_declared_dishonest(&self, path: &str) -> bool {
        self.declared_dishonest
            .iter()
            .any(|p| path == p || path.ends_with(p))
    }
}

#[cfg(test)]
mod tests {
    use super::TaxonomyCtx;

    #[test]
    fn test_usage() {
        let mut ctx = TaxonomyCtx::default();
        ctx.declared_dishonest.insert("SystemTime::now".to_string());
        assert!(ctx.is_declared_dishonest("std::time::SystemTime::now"));
        assert!(!ctx.is_declared_honest("std::time::SystemTime::now"));
        assert_eq!(ctx.honesty_depth, 1);
    }
}
