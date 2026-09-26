use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("mod_rs_purity") {
        checkers.push(Box::new(checkers::mod_rs_purity::ModRsPurity));
    }
}

// no test_usage necessary
