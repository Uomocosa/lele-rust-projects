use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, _config: &Config) {
    checkers.push(Box::new(checkers::no_super_imports::NoSuperImports));
}

// no test_usage necessary
