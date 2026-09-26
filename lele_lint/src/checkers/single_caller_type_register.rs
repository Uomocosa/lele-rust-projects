use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("single_caller_type") {
        checkers.push(Box::new(checkers::single_caller_type::SingleCallerType));
    }
}

// no test_usage necessary
