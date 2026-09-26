use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("test_inline") {
        checkers.push(Box::new(checkers::test_inline::TestInline));
    }
}

// no test_usage necessary
