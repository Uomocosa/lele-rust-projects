use bevy::prelude::*;

pub struct RemotePlugin {
    pub port: u16,
}

#[rustfmt::skip]
impl Plugin for RemotePlugin {
    fn build(&self, _app: &mut App) {
        crate::methods::clicker::remote::remote_plugin::build::build(_app)
    }
}
