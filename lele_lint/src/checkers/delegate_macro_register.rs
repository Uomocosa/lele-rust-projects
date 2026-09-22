use super::delegate_macro::DelegateMacro;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled(DelegateMacro::NAME) {
        checkers.push(Box::new(DelegateMacro));
    }
}

// no test_usage necessary
