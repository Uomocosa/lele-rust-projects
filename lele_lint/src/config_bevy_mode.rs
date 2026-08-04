// no test_usage necessary
use super::config::Config;

pub(crate) fn bevy_mode(config: &Config) -> bool {
    config
        .lele_lint
        .as_ref()
        .map(|s| s.bevy_mode)
        .unwrap_or(false)
}
