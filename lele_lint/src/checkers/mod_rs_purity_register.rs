use super::mod_rs_purity::ModRsPurity;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("mod_rs_purity") {
        checkers.push(Box::new(ModRsPurity));
    }
}

// no test_usage necessary
