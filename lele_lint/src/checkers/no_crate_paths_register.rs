use super::no_crate_paths::NoCratePaths;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("no_crate_paths") {
        checkers.push(Box::new(NoCratePaths));
    }
}

// no test_usage necessary
