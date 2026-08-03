// no test_usage necessary

use super::snake_case_files::SnakeCaseFiles;
use crate::checker::Checker;
use crate::config::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("snake_case_files") {
        checkers.push(Box::new(SnakeCaseFiles));
    }
}
