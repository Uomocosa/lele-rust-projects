// no test_usage necessary

use super::single_caller_type::SingleCallerType;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("single_caller_type") {
        checkers.push(Box::new(SingleCallerType));
    }
}
