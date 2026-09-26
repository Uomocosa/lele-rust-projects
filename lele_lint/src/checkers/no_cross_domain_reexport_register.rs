use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_cross_domain_reexport") {
        checkers.push(Box::new(
            checkers::no_cross_domain_reexport::NoCrossDomainReexport,
        ));
    }
}

// no test_usage necessary
