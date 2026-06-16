pub mod GamePluginMethod;
pub mod collect_input;
pub mod component;
#[path = "GamePlugin.rs"]
pub mod game_plugin;
pub mod system;

pub use collect_input::collect_input;
pub use game_plugin::GamePlugin;
