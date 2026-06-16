use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use bevy_p2p_app::clicker::component::{ClickCounter, ClickTarget};

pub fn create_headless_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.add_systems(Startup, spawn_click_target);
    app.add_systems(Update, simulate_click);
    app
}

fn spawn_click_target(mut commands: Commands) {
    commands.spawn((ClickTarget, ClickCounter { count: 0 }));
}

fn simulate_click(
    mut query: Query<&mut ClickCounter>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for mut counter in &mut query {
            counter.increment();
        }
    }
}

pub fn press_mouse_button(app: &mut App, button: MouseButton) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(button);
}

pub fn release_mouse_button(app: &mut App, button: MouseButton) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(button);
}

pub fn get_click_target_entity(app: &mut App) -> Option<Entity> {
    app.world_mut()
        .query_filtered::<Entity, With<ClickTarget>>()
        .iter(app.world())
        .next()
}

pub fn get_click_count(app: &mut App, entity: Entity) -> u32 {
    app.world()
        .get::<ClickCounter>(entity)
        .map(|c| c.count)
        .unwrap_or(0)
}

fn main() {
    let mut app = create_headless_app();
    app.update();
    tracing::info!("Headless app initialized - ClickTarget spawned");
    app.update();
    tracing::info!("Headless app running...");
}

#[cfg(test)]
mod tests {
    use crate::{create_headless_app, get_click_count, press_mouse_button, release_mouse_button};
    use bevy::input::mouse::MouseButton;
    use bevy::prelude::*;
    use bevy_p2p_app::clicker::component::ClickTarget;

    fn init_test_app() -> (App, Entity) {
        let mut app = create_headless_app();
        app.update();
        let entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ClickTarget>>()
            .iter(app.world())
            .collect();
        assert!(
            !entities.is_empty(),
            "ClickTarget should exist after startup"
        );
        (app, entities[0])
    }

    #[test]
    fn test_click_target_spawns() {
        let (mut app, entity) = init_test_app();
        let count = get_click_count(&mut app, entity);
        assert_eq!(count, 0, "Initial click count should be 0");
    }

    #[test]
    fn test_left_click_increments_counter() {
        let (mut app, entity) = init_test_app();

        press_mouse_button(&mut app, MouseButton::Left);
        app.update();

        let count = get_click_count(&mut app, entity);
        assert_eq!(count, 1, "Click should increment counter to 1");

        release_mouse_button(&mut app, MouseButton::Left);
    }

    #[test]
    fn test_multiple_clicks_increment_counter() {
        let (mut app, entity) = init_test_app();

        for _ in 0..5 {
            press_mouse_button(&mut app, MouseButton::Left);
            app.update();
            release_mouse_button(&mut app, MouseButton::Left);
        }

        let count = get_click_count(&mut app, entity);
        assert_eq!(count, 5, "5 clicks should increment counter to 5");
    }

    #[test]
    fn test_right_click_does_not_increment() {
        let (mut app, entity) = init_test_app();

        press_mouse_button(&mut app, MouseButton::Right);
        app.update();

        let count = get_click_count(&mut app, entity);
        assert_eq!(count, 0, "Right click should not increment counter");

        release_mouse_button(&mut app, MouseButton::Right);
    }
}
