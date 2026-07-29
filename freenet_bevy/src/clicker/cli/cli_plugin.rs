use bevy::prelude::*;

use super::cli_plugin_build;

pub struct Plugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { cli_plugin_build::build(app) }
}
