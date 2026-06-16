use bevy::input::ButtonInput;
use bevy::prelude::*;

use crate::boxes::collect_input;
use crate::boxes::component;

pub fn collect(
    mut query: Query<&mut component::PlayerInput>,
    button_input: Res<ButtonInput<KeyCode>>,
) {
    for mut player_input in &mut query {
        let input = collect_input(&button_input);
        player_input.input = input;
    }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component;
    use crate::boxes::system;
    use bevy::ecs::schedule::Schedule;
    use bevy::input::ButtonInput;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut world = World::new();

        world.spawn(component::PlayerInput::new());

        let mut button_input = ButtonInput::<KeyCode>::default();
        button_input.press(KeyCode::ArrowRight);

        let mut schedule = Schedule::default();
        schedule.add_systems(system::collect);

        world.insert_resource(button_input);
        schedule.run(&mut world);

        let mut query = world.query::<&component::PlayerInput>();
        let player_inputs: Vec<_> = query.iter(&world).collect();
        assert!(!player_inputs.is_empty());
        assert!(
            player_inputs[0].input.right,
            "Right key should be registered"
        );
    }
}
