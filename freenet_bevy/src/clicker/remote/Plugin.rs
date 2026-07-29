use bevy::prelude::*;

pub struct RemotePlugin {
    pub port: u16,
}

impl Plugin for RemotePlugin {
    fn build(&self, _app: &mut App) {
        crate::clicker::remote::PluginMethod::build::build(_app);
    }
}
