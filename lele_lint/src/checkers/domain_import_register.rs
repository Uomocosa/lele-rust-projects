// lele_lint: allow E001
// no test_usage necessary

use super::domain_import::DomainImport;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("domain_import") {
        checkers.push(Box::new(DomainImport));
    }
}
