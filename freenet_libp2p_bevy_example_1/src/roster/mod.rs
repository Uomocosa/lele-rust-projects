pub mod constants;
pub use constants::*;

pub mod peer_entry;
pub use peer_entry::PeerEntry;

pub mod roster_state;
pub use roster_state::RosterState;

pub mod merge_roster;
pub use merge_roster::merge_roster;

pub mod prune_stale;
pub use prune_stale::prune_stale;

pub mod decode_roster_update;
pub use decode_roster_update::decode_roster_update;

#[allow(clippy::module_inception)]
pub mod roster;
pub use roster::Roster;

pub mod event;
pub use event::Event;

pub mod roster_events;
pub use roster_events::RosterEvents;

pub mod config;
pub use config::Config;

mod config_new;
mod config_take_event_rx;

pub mod start_embedded_node;
pub use start_embedded_node::start_embedded_node;

pub mod setup_contract;
pub use setup_contract::setup_contract;

pub mod connect_and_run;
pub use connect_and_run::connect_and_run;

pub mod plugin;
pub use plugin::Plugin;

mod plugin_build;

pub mod bevy_systems;
