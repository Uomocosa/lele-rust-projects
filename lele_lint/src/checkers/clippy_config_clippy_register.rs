use super::clippy_config_clippy::ClippyConfigClippy;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("clippy_config_clippy") {
        checkers.push(Box::new(ClippyConfigClippy));
    }
}

// no test_usage necessary
