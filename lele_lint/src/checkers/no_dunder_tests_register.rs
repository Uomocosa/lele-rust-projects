use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_dunder_tests") {
        checkers.push(Box::new(checkers::no_dunder_tests::NoDunderTests));
    }
}

// no test_usage necessary
