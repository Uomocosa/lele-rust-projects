use super::root_reexport::RootReexport;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("root_reexport") {
        checkers.push(Box::new(RootReexport));
    }
}

// no test_usage necessary
