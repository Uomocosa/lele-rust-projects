use super::config::Config;
use crate::Dunder;

pub(crate) fn dunder(config: &Config) -> Dunder {
    let mut merged = Dunder::default();
    if let Some(section) = config.as_ref() {
        merged.folders.extend(section.dunder.folders.clone());
        merged.files.extend(section.dunder.files.iter().cloned());
    }
    merged
}

// no test_usage necessary
