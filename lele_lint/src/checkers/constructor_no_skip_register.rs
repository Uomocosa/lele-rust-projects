use super::constructor_no_skip::ConstructorNoSkip;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("constructor_no_skip") {
        checkers.push(Box::new(ConstructorNoSkip));
    }
}

// no test_usage necessary
