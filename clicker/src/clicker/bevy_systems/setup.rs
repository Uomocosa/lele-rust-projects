use crate::clicker;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::net_id;
pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    clicker::spawn_target(&mut commands, net_id::NetworkId(1), 0, true);
}
#[cfg(test)]
mod tests {
    use super::setup;
    use bevy::prelude::*;
    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Update, setup);
        app.update();
    }
}
