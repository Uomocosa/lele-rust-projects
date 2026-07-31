use bevy::prelude::*;

use super::cli_plugin_build;

pub struct CliPlugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for CliPlugin {
    fn build(&self, app: &mut App) { cli_plugin_build::build(app) }
}
