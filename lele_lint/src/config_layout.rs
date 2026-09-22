use super::config::Config;
use crate::Layout;

pub(crate) fn layout(config: &Config) -> Layout {
    config
        .as_ref()
        .and_then(|section| section.layout.as_deref())
        .map_or(Layout::Standard, Layout::parse)
}

// no test_usage necessary
