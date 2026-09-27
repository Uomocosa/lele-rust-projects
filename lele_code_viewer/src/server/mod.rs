mod events_stream;
pub use events_stream::events_stream;
mod router;
pub use router::router;
mod serve;
pub use serve::serve;

#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::structs::AppState;
