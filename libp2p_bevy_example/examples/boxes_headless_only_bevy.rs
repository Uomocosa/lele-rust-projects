use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use bevy_p2p_app::boxes::component::Player;
use bevy_p2p_app::boxes::component::{InputBuffer, PlayerInput, Position, Velocity};
use bevy_p2p_app::p2p::PlayerInputData;

const MOVE_SPEED: f32 = 200.0;
const JUMP_VELOCITY: f32 = 350.0;
const UP_SPEED: f32 = 150.0;
const GRAVITY: f32 = 800.0;
pub const GROUND_Y: f32 = -200.0;

pub fn create_headless_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.add_systems(Startup, spawn_player);
    app.add_systems(Update, collect_input.before(apply_physics));
    app.add_systems(Update, apply_physics.before(apply_position));
    app.add_systems(Update, apply_position);
    app
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player {
            peer_id: libp2p::PeerId::random(),
            is_local: true,
        },
        Position::new(0.0, GROUND_Y),
        Velocity::zero(),
        PlayerInput::new(),
        InputBuffer::default(),
        Transform::from_xyz(0.0, GROUND_Y, 0.0),
    ));
}

fn collect_input(mut query: Query<&mut PlayerInput>, button_input: Res<ButtonInput<KeyCode>>) {
    for mut player_input in &mut query {
        let left = button_input.pressed(KeyCode::KeyD);
        let right = false;
        let up = button_input.pressed(KeyCode::KeyW);
        let jump = button_input.pressed(KeyCode::Space);

        player_input.input = PlayerInputData::from_bools(left, right, up, jump);
    }
}

fn apply_physics(mut query: Query<(&mut Position, &mut Velocity, &PlayerInput)>, time: Res<Time>) {
    let dt = time.delta_secs();

    for (mut pos, mut vel, input) in &mut query {
        if input.input.left && !input.input.right {
            vel.x = -MOVE_SPEED;
        } else if input.input.right && !input.input.left {
            vel.x = MOVE_SPEED;
        } else {
            vel.x = 0.0;
        }

        if input.input.up {
            vel.y = UP_SPEED;
        } else if input.input.jump && pos.y <= GROUND_Y + 1.0 {
            vel.y = JUMP_VELOCITY;
        } else if pos.y > GROUND_Y {
            vel.y -= GRAVITY * dt;
        } else {
            vel.y = 0.0;
            pos.y = GROUND_Y;
        }

        pos.x += vel.x * dt;
        pos.y += vel.y * dt;

        if pos.y < GROUND_Y {
            pos.y = GROUND_Y;
            vel.y = 0.0;
        }
    }
}

fn apply_position(mut query: Query<(&Position, &mut Transform)>) {
    for (pos, mut transform) in &mut query {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

pub fn press_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
}

pub fn release_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
}

pub fn get_player_entity(app: &mut App) -> Option<Entity> {
    app.world_mut()
        .query_filtered::<Entity, With<Player>>()
        .iter(app.world())
        .next()
}

pub fn get_player_position(app: &mut App, entity: Entity) -> Vec3 {
    app.world()
        .get::<Transform>(entity)
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO)
}

pub fn reset_player(app: &mut App, entity: Entity) {
    if let Some(mut pos) = app.world_mut().get_mut::<Position>(entity) {
        pos.x = 0.0;
        pos.y = GROUND_Y;
    }
    if let Some(mut vel) = app.world_mut().get_mut::<Velocity>(entity) {
        vel.x = 0.0;
        vel.y = 0.0;
    }
}

pub const GROUND: f32 = GROUND_Y;

fn main() {
    let mut app = create_headless_app();
    app.update();
    tracing::info!("Headless app initialized - player spawned at ground level");
    app.update();
    tracing::info!("Headless app running...");
}

#[cfg(test)]
mod tests {
    use crate::{
        create_headless_app, get_player_position, press_key, release_key, reset_player, GROUND_Y,
    };
    use bevy::prelude::*;
    use bevy_p2p_app::boxes::component::Player;

    fn init_test_app() -> (App, Entity) {
        let mut app = create_headless_app();
        app.update();
        let entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .iter(app.world())
            .collect();
        assert!(!entities.is_empty(), "Player should exist after startup");
        (app, entities[0])
    }

    #[test]
    fn test_player_spawns_at_ground_level() {
        let (mut app, entity) = init_test_app();
        let position = get_player_position(&mut app, entity);
        assert!(
            (position.y - GROUND_Y).abs() < 0.1,
            "Player should spawn at ground level, got y={}",
            position.y
        );
    }

    #[test]
    fn test_w_key_moves_player_up() {
        let (mut app, entity) = init_test_app();
        reset_player(&mut app, entity);

        let pos_before = get_player_position(&mut app, entity);
        let y_before = pos_before.y;

        press_key(&mut app, KeyCode::KeyW);

        for _ in 0..10 {
            app.update();
        }

        let pos_after = get_player_position(&mut app, entity);

        release_key(&mut app, KeyCode::KeyW);

        assert!(
            pos_after.y > y_before,
            "W key should move player up: before_y={}, after_y={}",
            y_before,
            pos_after.y
        );
    }

    #[test]
    fn test_d_key_moves_player_left() {
        let (mut app, entity) = init_test_app();
        reset_player(&mut app, entity);

        let pos_before = get_player_position(&mut app, entity);
        let x_before = pos_before.x;

        press_key(&mut app, KeyCode::KeyD);

        for _ in 0..10 {
            app.update();
        }

        let pos_after = get_player_position(&mut app, entity);

        release_key(&mut app, KeyCode::KeyD);

        assert!(
            pos_after.x < x_before,
            "D key should move player left: before_x={}, after_x={}",
            x_before,
            pos_after.x
        );
    }

    #[test]
    fn test_space_key_causes_jump() {
        let (mut app, entity) = init_test_app();
        reset_player(&mut app, entity);

        let pos_before = get_player_position(&mut app, entity);
        let y_before = pos_before.y;

        press_key(&mut app, KeyCode::Space);

        for _ in 0..10 {
            app.update();
        }

        let pos_after = get_player_position(&mut app, entity);

        release_key(&mut app, KeyCode::Space);

        assert!(
            pos_after.y > y_before,
            "Space key should cause player to jump: before_y={}, after_y={}",
            y_before,
            pos_after.y
        );
    }

    #[test]
    fn test_jump_initiates_upward_movement() {
        let (mut app, entity) = init_test_app();
        reset_player(&mut app, entity);

        let pos_before = get_player_position(&mut app, entity);

        press_key(&mut app, KeyCode::Space);
        for _ in 0..20 {
            app.update();
        }
        release_key(&mut app, KeyCode::Space);

        let pos_after = get_player_position(&mut app, entity);

        assert!(
            pos_after.y > pos_before.y,
            "Space key should cause upward movement: before_y={}, after_y={}",
            pos_before.y,
            pos_after.y
        );
    }

    #[test]
    fn test_holding_w_counteracts_gravity() {
        let (mut app, entity) = init_test_app();
        reset_player(&mut app, entity);

        press_key(&mut app, KeyCode::Space);
        app.update();
        release_key(&mut app, KeyCode::Space);

        for _ in 0..10 {
            app.update();
        }
        let pos_before = get_player_position(&mut app, entity);
        let y_before = pos_before.y;

        press_key(&mut app, KeyCode::KeyW);
        for _ in 0..10 {
            app.update();
        }
        let pos_during = get_player_position(&mut app, entity);
        release_key(&mut app, KeyCode::KeyW);

        assert!(
            pos_during.y > y_before,
            "Holding W should counteract gravity: before_y={}, during_y={}",
            y_before,
            pos_during.y
        );
    }

    #[test]
    fn test_no_input_player_stays_at_ground() {
        let (mut app, entity) = init_test_app();
        reset_player(&mut app, entity);

        app.update();
        app.update();
        app.update();

        let pos = get_player_position(&mut app, entity);
        assert!(
            (pos.y - GROUND_Y).abs() < 0.1,
            "Without input, player should stay at ground: y={}",
            pos.y
        );
    }
}
