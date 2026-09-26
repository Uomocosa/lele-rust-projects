use crate::checkers;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("atomic_delegates") {
        checkers.push(Box::new(checkers::atomic_delegates::AtomicDelegates));
    }
}

// no test_usage necessary
