use crate::project;

pub fn default_settings() -> project::Settings {
    project::Settings {
        ignore: project::DEFAULT_IGNORE
            .iter()
            .map(|rule| (*rule).to_string())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::default_settings;

    #[test]
    fn test_usage() {
        let settings = default_settings();
        assert_eq!(settings.ignore.len(), 4);
        assert!(settings.ignore.iter().any(|rule| rule.contains("__OLD__")));
    }
}
