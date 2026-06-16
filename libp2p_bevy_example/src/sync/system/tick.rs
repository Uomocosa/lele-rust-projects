use bevy::prelude::*;

use crate::sync::resource;

pub fn tick(mut tick: ResMut<resource::Tick>) {
    tick.next();
}

#[cfg(test)]
mod tests {
    use crate::sync::resource;
    use crate::sync::system;
    use bevy::ecs::schedule::Schedule;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        world.init_resource::<resource::Tick>();
        let mut schedule = Schedule::default();
        schedule.add_systems(system::tick);
        schedule.run(&mut world);
        let tick = world.resource::<resource::Tick>();
        assert_eq!(tick.current, 1);
    }
}
