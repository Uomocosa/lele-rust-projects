#[path = "Role.rs"]
pub mod role;
pub use role::Role;

#[path = "Error.rs"]
pub mod error;
pub use error::ClientError;

#[path = "FreenetClient.rs"]
pub mod freenet_client;
pub use freenet_client::FreenetClient;

pub mod FreenetClientMethod;

pub mod clicker;

pub mod testing;
