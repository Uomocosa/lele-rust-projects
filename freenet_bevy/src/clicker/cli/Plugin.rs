use bevy::prelude::*;

pub struct CliPlugin;

impl Plugin for CliPlugin {
    fn build(&self, app: &mut App) {
        crate::clicker::cli::PluginMethod::build::build(app);
    }
}
