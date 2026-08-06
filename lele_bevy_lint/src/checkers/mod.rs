mod bevy_export;
mod bevy_export_check;
mod bevy_export_code;
mod bevy_export_name;
mod bevy_export_register;
mod bevy_folder;
mod bevy_folder_check;
mod bevy_folder_code;
mod bevy_folder_name;
mod bevy_folder_register;

use lele_lint::checker::Checker;
use lele_lint::config::Config;

pub fn build_checkers(config: &Config) -> Vec<Box<dyn Checker>> {
    let mut checkers: Vec<Box<dyn Checker>> = Vec::new();
    bevy_export::BevyExport::register(&mut checkers, config);
    bevy_folder::BevyFolder::register(&mut checkers, config);
    checkers
}

// no test_usage necessary
