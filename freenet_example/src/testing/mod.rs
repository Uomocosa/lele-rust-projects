#[path = "TestNode.rs"]
pub mod test_node;
pub use test_node::TestNode;

pub mod TestNodeMethod;
pub mod connect;
pub mod deploy;
pub mod drain;
pub mod get_count;
pub mod load_wasm;
pub mod recv_notification;
pub mod subscribe;
pub mod update_count;

pub use connect::connect;
pub use deploy::deploy;
pub use get_count::get_count;
pub use load_wasm::load_wasm;
pub use recv_notification::recv_notification;
pub use subscribe::subscribe;
pub use update_count::update_count;
