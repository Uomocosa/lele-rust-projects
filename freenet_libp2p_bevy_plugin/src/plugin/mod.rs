pub mod config;
pub use config::Config;

mod config_new;
mod config_take_event_rx;
mod config_take_roster_rx;

#[allow(clippy::module_inception)]
pub mod plugin;
pub use plugin::Plugin;

mod plugin_build;
