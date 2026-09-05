use super::method_file_co_location::MethodFileCoLocation;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("method_file_co_location") {
        checkers.push(Box::new(MethodFileCoLocation));
    }
}

// no test_usage necessary
