// no test_usage necessary

use super::no_cross_domain_reexport::NoCrossDomainReexport;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_cross_domain_reexport") {
        checkers.push(Box::new(NoCrossDomainReexport));
    }
}
