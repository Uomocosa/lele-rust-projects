use super::bevy_export;
use super::bevy_folder;
use lele_lint::checker::Checker;
use lele_lint::config::Config;

pub fn build_checkers(config: &Config) -> Vec<Box<dyn Checker>> {
    let mut checkers: Vec<Box<dyn Checker>> = Vec::new();
    bevy_export::BevyExport::register(&mut checkers, config);
    bevy_folder::BevyFolder::register(&mut checkers, config);
    checkers
}

// no test_usage necessary
