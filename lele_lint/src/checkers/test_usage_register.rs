use super::test_usage::TestUsage;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("test_usage") {
        checkers.push(Box::new(TestUsage));
    }
}

// no test_usage necessary
