use super::no_stuttered_type::NoStutteredType;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_stuttered_type") {
        checkers.push(Box::new(NoStutteredType));
    }
}

// no test_usage necessary
