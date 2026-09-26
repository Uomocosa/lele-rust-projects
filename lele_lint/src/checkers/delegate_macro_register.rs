use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled(checkers::delegate_macro::DelegateMacro::NAME) {
        checkers.push(Box::new(checkers::delegate_macro::DelegateMacro));
    }
}

// no test_usage necessary
