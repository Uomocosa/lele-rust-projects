use super::no_positional::NoPositional;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_positional") {
        checkers.push(Box::new(NoPositional));
    }
}

// no test_usage necessary
