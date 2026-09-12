use bevy::prelude::*;

use crate::clicker;

pub fn pos_log(
    cursors: Query<(&clicker::Owner, &clicker::PlayerNo, &Transform), With<clicker::CursorIcon>>,
    lobby: Res<clicker::ActiveLobby>,
    time: Res<Time>,
    mut last: Local<f64>,
) {
    let time = time.into_inner();
    let now = time.elapsed().as_secs_f64();
    if now - *last < 1.0 {
        return;
    }
    *last = now;
    let lobby = lobby.into_inner();
    for (_owner, player, transform) in &cursors {
        tracing::info!(
            "pos lobby={} player={} x={:.0} y={:.0}",
            **lobby,
            **player,
            transform.translation.x,
            transform.translation.y
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::pos_log;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        let cursor = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(net_id::NetworkId(1)),
                clicker::PlayerNo(1),
                Transform::from_translation(Vec3::new(10.0, 20.0, 10.0)),
            ))
            .id();
        app.add_systems(Update, pos_log);
        app.update();
        assert!(app.world().get::<clicker::PlayerNo>(cursor).is_some());
    }
}
