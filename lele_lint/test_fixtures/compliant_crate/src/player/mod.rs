mod player;
pub mod bevy_systems;

#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::enums::Event;
pub use player::Player;
