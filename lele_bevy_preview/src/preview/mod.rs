#[path = "__basic__/mod.rs"]
pub mod basic;
pub mod config;
pub use config::Config;

pub mod build;
pub mod distinct_colors;
pub mod ensure_visual;
pub mod install;
pub mod once;
pub mod render_states;
pub mod settled;
pub mod target_of;
pub mod warmup;

pub use build::build;
pub use ensure_visual::ensure_visual;
pub use install::install;
pub use once::once;
pub use render_states::render_states;
pub use settled::settled;
pub use target_of::target_of;
pub use warmup::warmup;
