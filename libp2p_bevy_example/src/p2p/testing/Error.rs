use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to get current executable path: {0}")]
    CurrentExe(std::io::Error),
    #[error("Failed to spawn child process: {0}")]
    SpawnChild(std::io::Error),
}
