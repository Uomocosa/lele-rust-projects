use std::path::Path;

use crate::Error;
use crate::project;

pub fn load_settings(path: &Path) -> Result<project::Settings, Error> {
    if !path.exists() {
        let defaults = project::default_settings();
        project::save_settings(path, &defaults)?;
        return Ok(defaults);
    }
    let text = std::fs::read_to_string(path)?;
    toml::from_str(&text).map_err(|err| Error::BadRequest(format!("{}: {err}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::load_settings;
    use crate::project;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("conf/settings.toml");
        let created = load_settings(&path).unwrap();
        assert_eq!(created, project::default_settings());
        assert!(path.exists());
        let custom = project::Settings {
            ignore: vec!["foo".to_string()],
        };
        project::save_settings(&path, &custom).unwrap();
        assert_eq!(load_settings(&path).unwrap(), custom);
    }
}
