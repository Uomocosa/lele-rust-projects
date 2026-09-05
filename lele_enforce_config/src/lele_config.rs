use std::path::Path;

use derive_more::Deref;

use super::lele_config_is_excluded;
use super::lele_config_load;
use crate::Error;
use crate::ExcludedPath;

#[derive(Debug, Clone, Default, Deref)]
pub struct LeleConfig {
    pub excluded_paths: Vec<ExcludedPath>,
}

#[rustfmt::skip]
impl LeleConfig {
    /// # Errors
    ///
    /// Returns [`Error`] when `lele.toml` exists but fails to parse as TOML.
    pub fn load(root: &Path) -> Result<Self, Error> { lele_config_load::load(root) }
    pub fn is_excluded(&self, root: &Path, path: &Path) -> bool { lele_config_is_excluded::is_excluded(self, root, path) }
}

#[cfg(test)]
mod tests {
    use super::LeleConfig;
    use std::path::Path;

    #[test]
    fn test_usage() {
        let config = LeleConfig::default();
        let root = Path::new("/root");
        assert!(config.excluded_paths.is_empty());
        assert!(!config.is_excluded(root, &root.join("foo")));
    }
}
