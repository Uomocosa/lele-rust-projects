use bevy::app::App;

use crate::sync::PluginMethod;

pub struct Plugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { PluginMethod::build(self, app) }
}
