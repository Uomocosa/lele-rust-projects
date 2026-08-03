// no test_usage necessary

use super::no_positional::NoPositional;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_positional") {
        checkers.push(Box::new(NoPositional));
    }
}
