use super::clippy_config_cargo::ClippyConfigCargo;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("clippy_config_cargo") {
        checkers.push(Box::new(ClippyConfigCargo));
    }
}

// no test_usage necessary
