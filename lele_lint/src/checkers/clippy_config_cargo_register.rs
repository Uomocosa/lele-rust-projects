use super::clippy_config_cargo::ClippyConfigCargo;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("clippy_config_cargo") {
        checkers.push(Box::new(ClippyConfigCargo));
    }
}

// no test_usage necessary
