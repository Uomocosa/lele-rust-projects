use super::domain_import::DomainImport;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("domain_import") {
        checkers.push(Box::new(DomainImport));
    }
}

// no test_usage necessary
