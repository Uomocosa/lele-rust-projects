use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Config(String),
    #[error("build failed: {0}")]
    Build(String),
    #[error("metadata failed: {0}")]
    Metadata(String),
    #[error("spawn failed: {0}")]
    Spawn(String),
    #[error("window error: {0}")]
    Window(String),
    #[error("telegram error: {0}")]
    Telegram(String),
    #[error("ffmpeg error: {0}")]
    Ffmpeg(String),
    #[error("assertion error: {0}")]
    Assertion(String),
    #[error("teardown error: {0}")]
    Teardown(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
