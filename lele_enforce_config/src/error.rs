use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("glob error: {0}")]
    Glob(String),
}
