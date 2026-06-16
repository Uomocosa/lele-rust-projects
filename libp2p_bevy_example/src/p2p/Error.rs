use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Noise handshake setup failed: {0}")]
    Noise(String),
    #[error("Gossipsub setup failed: {0}")]
    Gossipsub(String),
    #[error("mDNS initialization failed: {0}")]
    Mdns(String),
    #[error("Serialization failed: {0}")]
    Serialization(String),
}
