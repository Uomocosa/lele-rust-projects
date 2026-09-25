pub mod bevy_systems;
pub mod id;
pub mod link;
pub mod lobby_rooms;
pub mod room_peers;
pub mod session;

#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::constants::*;
pub use basic::enums::DiscoveryStatus;
pub use basic::messages::{Command, Event};
pub use basic::resources::{
    CommandSender, EventFeed, FreenetEndpoint, Multiplayer, MultiplayerFeed,
};

mod config;
pub use config::Config;

mod timing;
pub use timing::Timing;

mod error;
pub use error::Error;

mod game_token;

mod plugin;
pub use plugin::Plugin;
