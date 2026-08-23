pub mod click_delta;
pub use click_delta::ClickDelta;

pub mod owner;
pub use owner::Owner;

pub mod click_counter;
pub use click_counter::ClickCounter;

pub mod click_target;
pub use click_target::ClickTarget;

pub mod constants;
pub use constants::*;

pub mod spawn_target;
pub use spawn_target::spawn_target;

pub mod plugin;
pub use plugin::Plugin;

mod plugin_build;

pub mod bevy_systems;
