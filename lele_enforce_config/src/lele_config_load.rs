use std::path::Path;

use serde::Deserialize;

use crate::Error;
use crate::ExcludedPath;
use crate::LeleConfig;

#[derive(Deserialize)]
struct FileConfig {
    #[serde(default)]
    lele: LeleSection,
}

#[derive(Default, Deserialize)]
struct LeleSection {
    #[serde(default)]
    config: ConfigSection,
}

#[derive(Default, Deserialize)]
struct ConfigSection {
    #[serde(default)]
    exclude: Vec<String>,
}

/// # Errors
///
/// Returns [`Error::Config`] when `lele.toml` exists but fails to parse as TOML.
pub fn load(root: &Path) -> Result<LeleConfig, Error> {
    let path = root.join("lele.toml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Ok(LeleConfig::default()),
    };
    let parsed: FileConfig = toml::from_str(&content).map_err(|e| Error::Config(e.to_string()))?;
    for pattern in &parsed.lele.config.exclude {
        if let Err(e) = globset::Glob::new(pattern) {
            return Err(Error::Glob(e.to_string()));
        }
    }
    Ok(LeleConfig {
        excluded_paths: parsed
            .lele
            .config
            .exclude
            .into_iter()
            .map(ExcludedPath)
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn test_usage() {
        let root = std::path::Path::new("/tmp/nonexistent_lele_test_12345");
        let cfg = load(root).unwrap();
        assert!(cfg.excluded_paths.is_empty());
        assert!(!cfg.is_excluded(root, &root.join("foo")));
    }

    #[test]
    fn test_load_with_file() {
        let dir = tempfile::tempdir().unwrap();
        let content = "[lele.config]\nexclude = [\"foo\", \"bar\"]\n";
        std::fs::write(dir.path().join("lele.toml"), content).unwrap();
        let cfg = load(dir.path()).unwrap();
        assert!(cfg.is_excluded(dir.path(), &dir.path().join("foo")));
        assert!(cfg.is_excluded(dir.path(), &dir.path().join("bar")));
        assert!(!cfg.is_excluded(dir.path(), &dir.path().join("baz")));
    }

    #[test]
    fn test_load_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("lele.toml"), "[[[invalid").unwrap();
        assert!(load(dir.path()).is_err());
    }

    #[test]
    fn test_load_wrong_type() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("lele.toml"),
            "[lele.config]\nexclude = 42\n",
        )
        .unwrap();
        assert!(load(dir.path()).is_err());
    }

    #[test]
    fn test_load_invalid_glob() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("lele.toml"),
            "[lele.config]\nexclude = [\"[[[\"]\n",
        )
        .unwrap();
        assert!(load(dir.path()).is_err());
    }
}
