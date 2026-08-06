use lele_lint::checker::Checker;
use lele_lint::config::Config;

use super::bevy_folder::BevyFolder;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("bevy_folder") {
        checkers.push(Box::new(BevyFolder));
    }
}

// no test_usage necessary
