use bevy::prelude::*;

use crate::boxes::GamePluginMethod;

pub struct GamePlugin;

#[rustfmt::skip]
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) { GamePluginMethod::build(self, app) }
}
