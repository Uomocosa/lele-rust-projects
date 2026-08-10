use super::constructor_no_skip::ConstructorNoSkip;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("constructor_no_skip") {
        checkers.push(Box::new(ConstructorNoSkip));
    }
}

// no test_usage necessary
