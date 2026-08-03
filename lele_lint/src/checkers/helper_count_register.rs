// lele_lint: allow E001
// no test_usage necessary

use super::helper_count::HelperCount;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("helper_count") {
        checkers.push(Box::new(HelperCount));
    }
}
