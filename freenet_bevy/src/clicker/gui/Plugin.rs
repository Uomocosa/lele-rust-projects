use bevy::prelude::*;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        crate::clicker::gui::PluginMethod::build::build(app);
    }
}
