use bevy::prelude::*;

pub struct Plugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        crate::methods::clicker::cli::cli_plugin::build::build(app)
    }
}
