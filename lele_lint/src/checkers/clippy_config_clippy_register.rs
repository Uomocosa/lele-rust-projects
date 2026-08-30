use super::clippy_config_clippy::ClippyConfigClippy;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("clippy_config_clippy") {
        checkers.push(Box::new(ClippyConfigClippy));
    }
}

// no test_usage necessary
