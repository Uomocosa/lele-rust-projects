use crate::checkers;
use lele_lint::Checker;

pub fn build_checkers() -> Vec<Box<dyn Checker>> {
    let mut checkers: Vec<Box<dyn Checker>> = Vec::new();
    checkers::bevy_export::BevyExport::register(&mut checkers);
    checkers::bevy_folder::BevyFolder::register(&mut checkers);
    checkers::bevy_ui::BevyUi::register(&mut checkers);
    checkers::bevy_ui_mp4::BevyUiMp4::register(&mut checkers);
    checkers::bevy_plugin_scene::BevyPluginScene::register(&mut checkers);
    checkers::preview_routing::PreviewRouting::register(&mut checkers);
    checkers
}

// no test_usage necessary
