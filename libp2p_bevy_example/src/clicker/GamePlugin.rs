use bevy::prelude::*;

use crate::clicker::GamePluginMethod;

pub struct GamePlugin;

#[rustfmt::skip]
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) { GamePluginMethod::build(self, app) }
}
