use super::no_allow_attributes::NoAllowAttributes;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_allow_attributes") {
        checkers.push(Box::new(NoAllowAttributes));
    }
}

// no test_usage necessary
