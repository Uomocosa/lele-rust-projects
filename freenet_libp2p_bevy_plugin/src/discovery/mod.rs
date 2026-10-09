pub mod bevy_systems;
pub mod freenet;
pub mod libp2p;
pub mod state_machine;
#[cfg(feature = "default_ui")]
pub mod ui;

#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::enums::LinkStatus;
pub use basic::messages::{Command, Event};
pub use basic::newtypes::{EpochSecs, GameName, GameToken};
pub use basic::resources::{CommandSender, EventFeed, FreenetEndpoint, Snapshot, SnapshotFeed};
pub use basic::structs::{Channels, Member, Presence, Room, RoomRecord};
pub use basic::type_aliases::{Lobby, Members};

mod config;
pub use config::Config;

mod timing;
pub use timing::Timing;

mod error;
pub use error::Error;

mod game_token;

mod now_epoch;
pub use now_epoch::now_epoch;

mod run;
pub use run::run;

mod plugin;
pub use plugin::Plugin;

mod plugins;
pub use plugins::Plugins;
