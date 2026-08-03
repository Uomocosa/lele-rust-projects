// no test_usage necessary

use super::method_visibility::MethodVisibility;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("method_visibility") {
        checkers.push(Box::new(MethodVisibility));
    }
}
