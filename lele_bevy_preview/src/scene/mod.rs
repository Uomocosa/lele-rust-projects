#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::enums::Kind;
pub use basic::structs::{Scene, State, Timeline};
pub mod check;
pub use check::check;
