use std::path::Path;

use crate::Error;
use crate::config;

pub fn load_config(crate_dir: &Path) -> Result<config::PreviewConfig, Error> {
    let path = crate_dir.join(config::CONFIG_FILE);
    let text = std::fs::read_to_string(&path)?;
    parse(&text).map_err(|message| Error::Config(format!("{}: {message}", path.display())))
}

// needed helper: parse the [ui_preview] table out of a lele.toml body
fn parse(text: &str) -> Result<config::PreviewConfig, String> {
    let parsed: config::LeleToml = toml::from_str(text).map_err(|e| e.to_string())?;
    parsed
        .ui_preview
        .ok_or_else(|| "no [ui_preview] section".to_string())
}

#[cfg(test)]
mod tests {
    use super::load_config;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("lele.toml"),
            "[lele.lint]\n\n[ui_preview.bevy]\ncommand = [\"true\"]\nresource = \"Multiplayer\"\n",
        )
        .unwrap();
        let cfg = load_config(dir.path()).unwrap();
        assert_eq!(cfg.bevy.unwrap().resource, "Multiplayer");
        assert!(cfg.web.is_none());
    }

    #[test]
    fn test_missing_section_is_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("lele.toml"), "[lele.lint]\n").unwrap();
        assert!(load_config(dir.path()).is_err());
    }
}
