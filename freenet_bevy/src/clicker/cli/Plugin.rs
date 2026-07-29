use bevy::prelude::*;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        crate::clicker::cli::PluginMethod::build::build(app);
    }
}
