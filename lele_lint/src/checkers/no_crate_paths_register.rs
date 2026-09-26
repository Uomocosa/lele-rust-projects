use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_crate_paths") {
        checkers.push(Box::new(checkers::no_crate_paths::NoCratePaths));
    }
}

// no test_usage necessary
