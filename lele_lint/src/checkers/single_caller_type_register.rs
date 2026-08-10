use super::single_caller_type::SingleCallerType;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("single_caller_type") {
        checkers.push(Box::new(SingleCallerType));
    }
}

// no test_usage necessary
