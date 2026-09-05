use super::no_stuttered_path::NoStutteredPath;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_stuttered_path") {
        checkers.push(Box::new(NoStutteredPath));
    }
}

// no test_usage necessary
