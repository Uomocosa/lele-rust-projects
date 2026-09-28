use std::path::Path;

use crate::Error;
use crate::project;

pub fn save_settings(path: &Path, settings: &project::Settings) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = toml::to_string_pretty(settings).map_err(|err| Error::Server(err.to_string()))?;
    let temporary = path.with_extension("toml.tmp");
    std::fs::write(&temporary, text)?;
    std::fs::rename(&temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::save_settings;
    use crate::project;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        save_settings(&path, &project::default_settings()).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("__OLD__"));
        assert!(!dir.path().join("settings.toml.tmp").exists());
    }
}
