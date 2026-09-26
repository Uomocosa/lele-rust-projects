use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_trivial_accessors") {
        checkers.push(Box::new(checkers::no_trivial_accessors::NoTrivialAccessors));
    }
}

// no test_usage necessary
