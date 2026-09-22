use super::methods_layout::MethodsLayout;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled(MethodsLayout::NAME) {
        checkers.push(Box::new(MethodsLayout));
    }
}

// no test_usage necessary
