use std::path::PathBuf;

use crate::project;

pub fn settings_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(project::SETTINGS_DIR)
        .join(project::SETTINGS_FILE)
}

#[cfg(test)]
mod tests {
    use super::settings_path;

    #[test]
    fn test_usage() {
        let path = settings_path();
        assert!(path.ends_with("lele-code-viewer/settings.toml"));
    }
}
