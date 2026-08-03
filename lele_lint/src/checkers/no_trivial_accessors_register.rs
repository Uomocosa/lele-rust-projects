// no test_usage necessary

use super::no_trivial_accessors::NoTrivialAccessors;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_trivial_accessors") {
        checkers.push(Box::new(NoTrivialAccessors));
    }
}
