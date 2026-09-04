pub mod config;
pub use config::Config;

mod config_new;

pub mod constants;
pub use constants::*;

pub mod payload;
pub use payload::Payload;

pub mod player;
pub use player::Player;

pub mod local_player;
pub use local_player::LocalPlayer;

pub mod spawn_box;
pub use spawn_box::spawn_box;

pub mod pick_spawn_x;
pub use pick_spawn_x::pick_spawn_x;

pub mod spawn_x_for_player;
pub use spawn_x_for_player::spawn_x_for_player;

pub mod move_box;
pub use move_box::move_box;

pub mod jump_box;
pub use jump_box::jump_box;

pub mod plugin;
pub use plugin::Plugin;

mod plugin_build;

pub mod bevy_systems;
