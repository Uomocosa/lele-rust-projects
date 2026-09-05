use super::method_visibility::MethodVisibility;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("method_visibility") {
        checkers.push(Box::new(MethodVisibility));
    }
}

// no test_usage necessary
