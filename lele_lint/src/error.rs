use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no Cargo.toml found in {} or any parent directory", .0.display())]
    NoCargoRoot(std::path::PathBuf),

    #[error("src/ directory not found at {}", .0.display())]
    NoSrcDirectory(std::path::PathBuf),

    #[error("scan folder not found or not a directory: {}", .0.display())]
    NoScanFolder(std::path::PathBuf),

    #[error("methods layout requested but no `methods/` directory was found")]
    NoMethodsDir,

    #[error("filesystem error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("config error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("dunder path `{}` must end in `__<name>__`", .0.display())]
    NotDunder(std::path::PathBuf),
}
