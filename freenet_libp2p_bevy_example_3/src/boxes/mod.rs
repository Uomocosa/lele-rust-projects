pub mod config;
pub use config::Config;

mod config_new;

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

pub mod pick_spawn_x;
pub use pick_spawn_x::pick_spawn_x;

pub mod spawn_x_for_player;
pub use spawn_x_for_player::spawn_x_for_player;

pub mod latest_snapshot;
pub use latest_snapshot::LatestSnapshot;

pub mod sim_state;
pub use sim_state::SimState;

pub mod netcode_lockstep;
pub use netcode_lockstep::NetcodeLockstep;

pub mod plugin;
pub use plugin::Plugin;

mod plugin_build;

pub mod bevy_systems;
