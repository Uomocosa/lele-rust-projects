mod enemy;
mod enemy_spawn;
mod tick_enemies;           // VIOLATION: bevy system func outside bevy_systems/

pub use enemy::Enemy;
pub use tick_enemies::tick_enemies;
