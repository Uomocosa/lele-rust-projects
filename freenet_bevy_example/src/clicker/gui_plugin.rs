use bevy::prelude::*;

use super::gui_plugin_build;

pub struct GuiPlugin;

#[rustfmt::skip]
impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) { gui_plugin_build::build(app) }
}
