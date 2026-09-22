use super::no_comments::NoComments;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled(NoComments::NAME) {
        checkers.push(Box::new(NoComments));
    }
}

// no test_usage necessary
