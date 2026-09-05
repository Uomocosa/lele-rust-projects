use super::helper_count::HelperCount;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("helper_count") {
        checkers.push(Box::new(HelperCount));
    }
}

// no test_usage necessary
