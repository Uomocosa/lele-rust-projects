use super::no_positional::NoPositional;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("no_positional") {
        checkers.push(Box::new(NoPositional));
    }
}

// no test_usage necessary
