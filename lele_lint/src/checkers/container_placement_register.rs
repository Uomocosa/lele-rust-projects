use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("container_placement") {
        checkers.push(Box::new(checkers::container_placement::ContainerPlacement));
    }
}

// no test_usage necessary
