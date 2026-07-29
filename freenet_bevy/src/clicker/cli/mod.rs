#[path = "Plugin.rs"]
pub mod plugin;
pub use plugin::Plugin;

#[path = "CliCommand.rs"]
pub mod cli_command;
pub use cli_command::CliCommand;

pub mod CliCommandMethod;
pub mod PluginMethod;
pub mod system;
