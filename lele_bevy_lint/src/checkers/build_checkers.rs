use crate::checkers;
use lele_lint::Checker;

pub fn build_checkers() -> Vec<Box<dyn Checker>> {
    let mut checkers: Vec<Box<dyn Checker>> = Vec::new();
    checkers::bevy_export::BevyExport::register(&mut checkers);
    checkers::bevy_folder::BevyFolder::register(&mut checkers);
    checkers::bevy_ui::BevyUi::register(&mut checkers);
    checkers
}

// no test_usage necessary
