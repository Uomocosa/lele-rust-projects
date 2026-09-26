use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled(checkers::no_comments::NoComments::NAME) {
        checkers.push(Box::new(checkers::no_comments::NoComments));
    }
}

// no test_usage necessary
