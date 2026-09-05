use super::test_inline::TestInline;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("test_inline") {
        checkers.push(Box::new(TestInline));
    }
}

// no test_usage necessary
