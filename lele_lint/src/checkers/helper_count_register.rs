use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("helper_count") {
        checkers.push(Box::new(checkers::helper_count::HelperCount));
    }
}

// no test_usage necessary
