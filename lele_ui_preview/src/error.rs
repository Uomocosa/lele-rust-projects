use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("config error: {0}")]
    Config(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("chrome devtools error: {0}")]
    Cdp(String),
    #[error("bevy remote protocol error: {0}")]
    Brp(String),
    #[error("image error: {0}")]
    Image(String),
    #[error("timed out waiting for {0}")]
    Timeout(String),
}

// no test_usage necessary
