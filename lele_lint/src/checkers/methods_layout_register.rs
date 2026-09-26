use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled(checkers::methods_layout::MethodsLayout::NAME) {
        checkers.push(Box::new(checkers::methods_layout::MethodsLayout));
    }
}

// no test_usage necessary
