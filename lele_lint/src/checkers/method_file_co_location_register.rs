use super::method_file_co_location::MethodFileCoLocation;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("method_file_co_location") {
        checkers.push(Box::new(MethodFileCoLocation));
    }
}

// no test_usage necessary
