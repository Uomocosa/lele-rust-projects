use super::domain_import::DomainImport;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("domain_import") {
        checkers.push(Box::new(DomainImport));
    }
}

// no test_usage necessary
