use super::helper_count::HelperCount;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("helper_count") {
        checkers.push(Box::new(HelperCount));
    }
}

// no test_usage necessary
