pub mod GamePluginMethod;
pub mod component;
#[path = "GamePlugin.rs"]
pub mod game_plugin;
pub mod system;

pub use game_plugin::GamePlugin;
