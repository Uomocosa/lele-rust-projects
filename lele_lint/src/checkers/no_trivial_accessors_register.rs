use super::no_trivial_accessors::NoTrivialAccessors;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("no_trivial_accessors") {
        checkers.push(Box::new(NoTrivialAccessors));
    }
}

// no test_usage necessary
