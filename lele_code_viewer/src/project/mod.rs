mod discover_projects;
pub use discover_projects::discover_projects;
mod index_for;
pub use index_for::index_for;

#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::structs::{ProjectRef, Registry};
