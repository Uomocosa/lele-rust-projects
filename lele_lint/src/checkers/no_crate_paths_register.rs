use super::no_crate_paths::NoCratePaths;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_crate_paths") {
        checkers.push(Box::new(NoCratePaths));
    }
}

// no test_usage necessary
