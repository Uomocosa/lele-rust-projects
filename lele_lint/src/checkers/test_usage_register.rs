use super::test_usage::TestUsage;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("test_usage") {
        checkers.push(Box::new(TestUsage));
    }
}

// no test_usage necessary
