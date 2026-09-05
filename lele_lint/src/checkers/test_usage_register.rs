use super::test_usage::TestUsage;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("test_usage") {
        checkers.push(Box::new(TestUsage));
    }
}

// no test_usage necessary
