pub mod player_id;
pub use player_id::PlayerId;

pub mod player;
pub use player::Player;

pub mod local_player;
pub use local_player::LocalPlayer;

pub mod constants;
pub use constants::*;

pub mod spawn_box;
pub use spawn_box::spawn_box;

pub mod move_box;
pub use move_box::move_box;

pub mod jump_box;
pub use jump_box::jump_box;

pub mod plugin;
pub use plugin::Plugin;

mod plugin_build;

pub mod bevy_systems;
