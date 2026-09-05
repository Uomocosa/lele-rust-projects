use super::atomic_delegates::AtomicDelegates;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("atomic_delegates") {
        checkers.push(Box::new(AtomicDelegates));
    }
}

// no test_usage necessary
