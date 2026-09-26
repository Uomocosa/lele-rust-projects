use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("constructor_no_skip") {
        checkers.push(Box::new(checkers::constructor_no_skip::ConstructorNoSkip));
    }
}

// no test_usage necessary
