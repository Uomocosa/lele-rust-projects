use std::path::Path;

use globset::Glob;
use globset::GlobSetBuilder;

use super::lele_config::LeleConfig;

pub fn is_excluded(config: &LeleConfig, root: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut builder = GlobSetBuilder::new();
    for excluded in &config.excluded_paths {
        if let Ok(glob) = Glob::new(excluded.as_str()) {
            builder.add(glob);
        }
    }
    builder.build().is_ok_and(|set| set.is_match(relative))
}

#[cfg(test)]
mod tests {
    use super::is_excluded;
    use crate::ExcludedPath;
    use crate::LeleConfig;

    fn config_with(patterns: &[&str]) -> LeleConfig {
        LeleConfig {
            excluded_paths: patterns
                .iter()
                .map(|p| ExcludedPath((*p).to_string()))
                .collect(),
        }
    }

    #[test]
    fn test_usage() {
        let root = std::path::Path::new("/root");
        let config = config_with(&["target"]);
        assert!(is_excluded(&config, root, &root.join("target")));
        assert!(!is_excluded(&config, root, &root.join("src")));
    }

    #[test]
    fn test_nested_glob() {
        let root = std::path::Path::new("/root");
        let config = config_with(&["*/foo/bar/*"]);
        assert!(is_excluded(&config, root, &root.join("x/foo/bar/y")));
        assert!(!is_excluded(&config, root, &root.join("x/foo/baz/y")));
    }

    #[test]
    fn test_bare_name_does_not_match_nested() {
        let root = std::path::Path::new("/root");
        let config = config_with(&["target"]);
        assert!(!is_excluded(&config, root, &root.join("x/target")));
    }
}
