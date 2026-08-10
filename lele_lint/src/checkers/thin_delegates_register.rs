use super::thin_delegates::ThinDelegates;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("thin_delegates") {
        checkers.push(Box::new(ThinDelegates));
    }
}

// no test_usage necessary
