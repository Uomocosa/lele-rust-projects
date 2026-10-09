use crate::common;
use crate::Config;
use crate::DunderPath;

pub fn dunder_paths(config: &Config) -> Vec<DunderPath> {
    let mut merged = common::default_dunder_paths();
    if let Some(section) = config.as_ref() {
        merged.extend(section.lint.dunder_paths.iter().cloned());
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::dunder_paths;
    use crate::Config;
    use crate::LeleSection;

    #[test]
    fn test_usage() {
        let defaults = dunder_paths(&Config::default());
        assert_eq!(defaults.len(), 1);
        assert_eq!(defaults[0].module_name(), "basic");
    }

    #[test]
    fn test_usage_appends_configured_dunder_paths() {
        let section: LeleSection =
            toml::from_str("[lint]\ndunder_paths = [\"__prelude__.rs\"]\n").unwrap();
        let merged = dunder_paths(&Config(Some(section)));
        assert_eq!(merged.len(), 2);
        assert!(merged[1].is_dunder_file());
    }

    #[test]
    fn test_usage_rejects_non_dunder_config() {
        assert!(toml::from_str::<LeleSection>("[lint]\ndunder_paths = [\"basic\"]\n").is_err());
        assert!(toml::from_str::<LeleSection>("[lint]\ndunder_whitelist = {}\n").is_err());
    }
}
