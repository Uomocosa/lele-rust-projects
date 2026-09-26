use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("constants_placement") {
        checkers.push(Box::new(checkers::constants_placement::ConstantsPlacement));
    }
}

// no test_usage necessary
