use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("single_field_newtype") {
        checkers.push(Box::new(checkers::single_field_newtype::SingleFieldNewtype));
    }
}

// no test_usage necessary
