use crate::Config;
use crate::Dunder;

pub fn dunder(config: &Config) -> Dunder {
    let mut merged = Dunder::default();
    if let Some(section) = config.as_ref() {
        merged
            .folders
            .extend(section.dunder_whitelist.folders.clone());
        merged
            .files
            .extend(section.dunder_whitelist.files.iter().cloned());
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::dunder;
    use crate::Config;

    #[test]
    fn test_usage() {
        let dunder = dunder(&Config::default());
        assert!(dunder.folders.contains_key("__basic__"));
    }
}
