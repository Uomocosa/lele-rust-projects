use super::container_placement::ContainerPlacement;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("container_placement") {
        checkers.push(Box::new(ContainerPlacement));
    }
}

// no test_usage necessary
