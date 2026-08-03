// lele_lint: allow E001
// no test_usage necessary

use super::constructor_no_skip::ConstructorNoSkip;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("constructor_no_skip") {
        checkers.push(Box::new(ConstructorNoSkip));
    }
}
