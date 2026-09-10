use lele_lint::checker::Checker;
use lele_lint::config::Config;

use super::bevy_ui::BevyUi;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("bevy_ui") {
        checkers.push(Box::new(BevyUi));
    }
}

// no test_usage necessary
