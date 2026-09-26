use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("snake_case_files") {
        checkers.push(Box::new(checkers::snake_case_files::SnakeCaseFiles));
    }
}

// no test_usage necessary
