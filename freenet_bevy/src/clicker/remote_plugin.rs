use bevy::prelude::*;

use super::remote_plugin_build;

pub struct RemotePlugin {
    pub port: u16,
}

#[rustfmt::skip]
impl Plugin for RemotePlugin {
    fn build(&self, _app: &mut App) { remote_plugin_build::build(_app) }
}
