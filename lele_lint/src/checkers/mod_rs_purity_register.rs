use super::mod_rs_purity::ModRsPurity;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("mod_rs_purity") {
        checkers.push(Box::new(ModRsPurity));
    }
}

// no test_usage necessary
