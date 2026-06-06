use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Freenet configuration error: {0}")]
    Config(String),
    #[error("Freenet runtime error: {0}")]
    Runtime(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
