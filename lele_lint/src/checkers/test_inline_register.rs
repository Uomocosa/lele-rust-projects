use super::test_inline::TestInline;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("test_inline") {
        checkers.push(Box::new(TestInline));
    }
}

// no test_usage necessary
