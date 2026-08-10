use super::snake_case_files::SnakeCaseFiles;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("snake_case_files") {
        checkers.push(Box::new(SnakeCaseFiles));
    }
}

// no test_usage necessary
