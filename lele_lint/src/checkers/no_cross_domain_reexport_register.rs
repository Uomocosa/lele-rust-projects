use super::no_cross_domain_reexport::NoCrossDomainReexport;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("no_cross_domain_reexport") {
        checkers.push(Box::new(NoCrossDomainReexport));
    }
}

// no test_usage necessary
