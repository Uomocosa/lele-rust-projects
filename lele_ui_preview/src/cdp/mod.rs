#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::structs::{Browser, Session};

mod call;
pub use call::call;
mod evaluate;
pub use evaluate::evaluate;
mod launch_chrome;
pub use launch_chrome::launch_chrome;
mod navigate;
pub use navigate::navigate;
mod open_page;
pub use open_page::open_page;
mod press_key;
pub use press_key::press_key;
mod screenshot;
pub use screenshot::screenshot;
mod set_viewport;
pub use set_viewport::set_viewport;
