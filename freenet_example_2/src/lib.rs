#![allow(clippy::result_large_err)]

pub mod role;
pub use role::Role;

pub mod client_error;
pub use client_error::ClientError;

pub mod clicker_error;
pub use clicker_error::ClickerError;

pub mod clicker_client;
pub use clicker_client::ClickerClient;

pub mod clicker_client_method;

pub mod set_client;
pub use set_client::SetClient;

pub mod set_client_method;

pub mod freenet_client;
pub use freenet_client::FreenetClient;

pub mod freenet_client_method;

pub(crate) mod recv_after_get;
pub(crate) use recv_after_get::recv_after_get;

pub(crate) mod recv_response;
pub(crate) use recv_response::recv_response;

// The `testing` module is intentionally public so that example binaries
// and integration tests can reuse TestNode and test helpers.
pub mod testing;
