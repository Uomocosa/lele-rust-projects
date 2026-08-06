use super::test_inline::TestInline;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("test_inline") {
        checkers.push(Box::new(TestInline));
    }
}

// no test_usage necessary
