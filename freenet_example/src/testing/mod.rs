#[path = "TestNode.rs"]
pub(crate) mod test_node;
pub(crate) use test_node::TestNode;

pub(crate) mod TestNodeMethod;
pub(crate) mod load_wasm;
pub(crate) mod connect;
pub(crate) mod deploy;
pub(crate) mod subscribe;
pub(crate) mod get_count;
pub(crate) mod update_count;
pub(crate) mod recv_notification;
pub(crate) mod drain;

pub(crate) use load_wasm::load_wasm;
pub(crate) use connect::connect;
pub(crate) use deploy::deploy;
pub(crate) use subscribe::subscribe;
pub(crate) use get_count::get_count;
pub(crate) use update_count::update_count;
pub(crate) use recv_notification::recv_notification;
