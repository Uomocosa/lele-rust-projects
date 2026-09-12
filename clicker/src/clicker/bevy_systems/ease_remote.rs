use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

const SNAP_SQ: f32 = 40000.0;
const EASE_RATE: f32 = 12.0;

pub fn ease_remote(
    time: Res<Time>,
    mut remotes: Query<
        (&clicker::Owner, &mut Transform, &clicker::TargetPos),
        With<clicker::CursorIcon>,
    >,
    own: Res<net_id::NetworkId>,
) {
    let time = time.into_inner();
    let own = own.into_inner();
    let dt = time.delta_secs();
    let blend = 1.0 - (-EASE_RATE * dt).exp();
    for (owner, mut transform, target) in &mut remotes {
        if ***owner == **own {
            continue;
        }
        let current = Vec2::new(transform.translation.x, transform.translation.y);
        let goal: Vec2 = **target;
        if current.distance_squared(goal) > SNAP_SQ {
            transform.translation.x = goal.x;
            transform.translation.y = goal.y;
            continue;
        }
        let next = current.lerp(goal, blend);
        transform.translation.x = next.x;
        transform.translation.y = next.y;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::ease_remote;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(net_id::NetworkId(1));
        let remote = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(net_id::NetworkId(2)),
                clicker::TargetPos(Vec2::new(500.0, 500.0)),
                Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            ))
            .id();
        let local = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(net_id::NetworkId(1)),
                clicker::TargetPos(Vec2::new(500.0, 500.0)),
                Transform::from_translation(Vec3::new(7.0, 7.0, 10.0)),
            ))
            .id();
        app.add_systems(Update, ease_remote);
        app.update();
        let snapped = app.world().get::<Transform>(remote).unwrap().translation;
        assert!((snapped.x - 500.0).abs() < 0.001);
        assert!((snapped.y - 500.0).abs() < 0.001);
        let kept = app.world().get::<Transform>(local).unwrap().translation;
        assert!((kept.x - 7.0).abs() < 0.001);
    }
}
