pub mod cli;
pub mod gui;
pub mod remote;

pub mod command;
pub mod count_changed;
pub mod error;
pub mod event;
pub mod increment;
pub mod plugin;
mod plugin_build;
pub mod poll_freenet_events;
pub mod state;

pub use command::ClickerCommand;
pub use count_changed::CountChanged;
pub use error::ClickerError;
pub use event::ClickerEvent;
pub use increment::increment;
pub use plugin::{ClickerConfig, ClickerPlugin};
pub use poll_freenet_events::poll_freenet_events;
pub use state::ClickerState;
