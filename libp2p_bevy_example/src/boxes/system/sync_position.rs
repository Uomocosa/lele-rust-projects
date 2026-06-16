use bevy::prelude::*;
use tracing;

use crate::boxes::component;

pub fn sync_position(mut query: Query<(&component::Position, &mut Transform)>) {
    for (pos, mut transform) in &mut query {
        tracing::trace!(target: "position_sync", x = pos.x, y = pos.y);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component;
    use crate::boxes::system;
    use bevy::ecs::schedule::Schedule;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut world = World::new();

        world.spawn((component::Position::new(100.0, 200.0), Transform::default()));

        let mut schedule = Schedule::default();
        schedule.add_systems(system::sync_position);
        schedule.run(&mut world);

        let mut query = world.query::<&Transform>();
        let transforms: Vec<_> = query.iter(&world).collect();
        assert!(!transforms.is_empty());
        assert_eq!(transforms[0].translation.x, 100.0);
        assert_eq!(transforms[0].translation.y, 200.0);
    }
}
