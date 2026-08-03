// lele_lint: allow E001
// no test_usage necessary

use super::thin_delegates::ThinDelegates;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("thin_delegates") {
        checkers.push(Box::new(ThinDelegates));
    }
}
