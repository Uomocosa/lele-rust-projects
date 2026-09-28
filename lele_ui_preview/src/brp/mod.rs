#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::constants::*;
pub use basic::structs::Fixture;

mod call_brp;
pub use call_brp::call_brp;
mod crawl_bevy;
pub use crawl_bevy::crawl_bevy;
mod fixtures;
pub use fixtures::fixtures;
