mod events_stream;
pub use events_stream::events_stream;
mod project_list;
pub use project_list::project_list;
mod rescan;
pub use rescan::rescan;
mod router;
pub use router::router;
mod serve;
pub use serve::serve;
mod start_self_update;
pub use start_self_update::start_self_update;

#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::structs::{AppState, ServeOptions};
