use super::atomic_file::AtomicFile;
use crate::checker;
use crate::config;

pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
    if config.checker_enabled("atomic_file") {
        checkers.push(Box::new(AtomicFile));
    }
}

// no test_usage necessary
