use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn setup(
    mut commands: Commands,
    own: Res<net_id::NetworkId>,
    lobby: Res<clicker::ActiveLobby>,
) {
    let own = own.into_inner();
    let lobby = lobby.into_inner();
    commands.spawn(Camera2d);
    clicker::spawn_target(&mut commands, *own, 0, 1, true);
    commands.spawn((
        clicker::OwnScore,
        Text2d::new("you: 0"),
        Transform::from_translation(Vec3::new(0.0, 180.0, 1.0)),
    ));
    commands.spawn((
        clicker::GlobalScore,
        Text2d::new(format!("lobby {} global: 0", **lobby)),
        Transform::from_translation(Vec3::new(0.0, 150.0, 1.0)),
    ));
}

#[cfg(test)]
mod tests {
    use super::setup;
    use crate::clicker;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.add_systems(Update, setup);
        app.update();
        let scores = app.world_mut().query::<&Text2d>().iter(app.world()).count();
        assert_eq!(scores, 2);
    }
}
