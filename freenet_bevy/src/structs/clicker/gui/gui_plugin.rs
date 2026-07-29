use bevy::prelude::*;

pub struct GuiPlugin;

#[rustfmt::skip]
impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        crate::methods::clicker::gui::gui_plugin::build::build(app)
    }
}
