use super::constants_placement::ConstantsPlacement;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("constants_placement") {
        checkers.push(Box::new(ConstantsPlacement));
    }
}

// no test_usage necessary
