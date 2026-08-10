use super::config::Config;

pub(crate) fn checker_enabled(config: &Config, name: &str) -> bool {
    config
        .as_ref()
        .and_then(|s| s.checkers.get(name))
        .copied()
        .unwrap_or(true)
}

// no test_usage necessary
