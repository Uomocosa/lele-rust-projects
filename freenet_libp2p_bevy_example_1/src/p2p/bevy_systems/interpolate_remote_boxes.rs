use bevy::prelude::*;

use crate::boxes;
use crate::p2p;

type RemoteBoxQuery<'w, 's, 'a> = Query<
    'w,
    's,
    (Entity, &'a mut Transform, &'a mut p2p::RemoteTarget),
    (With<boxes::Player>, Without<boxes::LocalPlayer>),
>;

pub fn interpolate_remote_boxes(
    mut commands: Commands,
    mut query: RemoteBoxQuery<'_, '_, '_>,
    time: Res<Time>,
) {
    let window_secs = p2p::constants::INTERPOLATION_WINDOW_MS as f32 / 1000.0;
    let step = (time.delta_secs() / window_secs).min(1.0);

    for (entity, mut transform, target) in &mut query {
        let current = transform.translation.truncate();
        let to_target = target.pos - current;
        if to_target.length() < 1.0 {
            transform.translation = target.pos.extend(transform.translation.z);
            commands.entity(entity).remove::<p2p::RemoteTarget>();
            continue;
        }
        let next = current + to_target * step;
        transform.translation = next.extend(transform.translation.z);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::interpolate_remote_boxes;
    use crate::boxes;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(50),
        ));
        app.add_plugins(MinimalPlugins);
        let entity = app
            .world_mut()
            .spawn((
                boxes::Player(boxes::PlayerId(7)),
                Transform::from_xyz(0.0, 0.0, 0.0),
                p2p::RemoteTarget {
                    pos: Vec2::new(10.0, 0.0),
                    tick: 1,
                },
            ))
            .id();
        app.add_systems(Update, interpolate_remote_boxes);

        app.update();
        app.update();

        let transform = app.world().get::<Transform>(entity);
        assert!(transform.is_some());
        assert!(transform.is_some_and(|t| t.translation.x > 0.0 && t.translation.x < 10.0));
    }
}
