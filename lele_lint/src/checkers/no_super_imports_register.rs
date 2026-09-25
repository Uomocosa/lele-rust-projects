use super::no_super_imports::NoSuperImports;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, _config: &Config) {
    checkers.push(Box::new(NoSuperImports));
}

// no test_usage necessary
