#[path = "Plugin.rs"]
pub mod plugin;
pub use plugin::GuiPlugin;

pub mod PluginMethod;
pub mod component;
pub mod system;
