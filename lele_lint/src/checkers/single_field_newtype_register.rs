use super::single_field_newtype::SingleFieldNewtype;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("single_field_newtype") {
        checkers.push(Box::new(SingleFieldNewtype));
    }
}

// no test_usage necessary
