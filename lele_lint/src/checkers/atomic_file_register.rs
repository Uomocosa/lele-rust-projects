use super::atomic_file::AtomicFile;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("atomic_file") {
        checkers.push(Box::new(AtomicFile));
    }
}

// no test_usage necessary
