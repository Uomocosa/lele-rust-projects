#[path = "Plugin.rs"]
pub mod plugin;
pub use plugin::{ClickerConfig, ClickerPlugin};

#[path = "ClickerCommand.rs"]
pub mod clicker_command;
pub use clicker_command::ClickerCommand;

#[path = "ClickerEvent.rs"]
pub mod clicker_event;
pub use clicker_event::ClickerEvent;

#[path = "ClickerError.rs"]
pub mod clicker_error;
pub use clicker_error::ClickerError;

pub mod PluginMethod;
pub mod component;
pub mod headless;
pub mod message;
pub mod resource;
pub mod system;
