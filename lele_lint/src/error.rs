use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no Cargo.toml found in {0} or any parent directory")]
    NoCargoRoot(String),

    #[error("src/ directory not found at {0}")]
    NoSrcDirectory(String),

    #[error("scan folder not found or not a directory: {0}")]
    NoScanFolder(String),

    #[error("filesystem error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("config error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
