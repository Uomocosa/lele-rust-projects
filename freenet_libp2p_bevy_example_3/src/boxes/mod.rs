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

pub mod latest_snapshot;
pub use latest_snapshot::LatestSnapshot;

pub mod predicted_snapshot;
pub use predicted_snapshot::PredictedSnapshot;

pub mod sim_state;
pub use sim_state::SimState;

pub mod netcode_lockstep;
pub use netcode_lockstep::NetcodeLockstep;

pub mod plugin;
pub use plugin::Plugin;

mod plugin_build;

pub mod bevy_systems;
