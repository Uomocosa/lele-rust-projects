use super::no_trivial_accessors::NoTrivialAccessors;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_trivial_accessors") {
        checkers.push(Box::new(NoTrivialAccessors));
    }
}

// no test_usage necessary
