#[path = "Role.rs"]
pub mod role;
pub use role::Role;

#[path = "Error.rs"]
pub mod error;
pub use error::ClientError;

#[path = "ClickerError.rs"]
pub mod clicker_error;
pub use clicker_error::ClickerError;

#[path = "ClickerClient.rs"]
pub mod clicker_client;
pub use clicker_client::ClickerClient;

pub mod ClickerClientMethod;

#[path = "FreenetClient.rs"]
pub mod freenet_client;
pub use freenet_client::FreenetClient;

pub mod FreenetClientMethod;

pub(crate) mod recv_after_get;
pub(crate) use recv_after_get::recv_after_get;

pub(crate) mod recv_response;
pub(crate) use recv_response::recv_response;

#[cfg(test)]
pub(crate) mod testing;
