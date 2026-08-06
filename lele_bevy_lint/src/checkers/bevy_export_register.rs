use lele_lint::checker::Checker;
use lele_lint::config::Config;

use super::bevy_export::BevyExport;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("bevy_export") {
        checkers.push(Box::new(BevyExport));
    }
}

// no test_usage necessary
