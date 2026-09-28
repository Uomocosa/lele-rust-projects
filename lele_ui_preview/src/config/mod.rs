#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::constants::*;
pub use basic::structs::{BevyConfig, LeleToml, PreviewConfig, Viewport, WebConfig};

mod load_config;
pub use load_config::load_config;
