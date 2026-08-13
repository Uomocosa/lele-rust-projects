use thiserror::Error;

use crate::error_method;
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to run gh: {0}")]
    GhSpawn(#[source] std::io::Error),
    #[error("gh command failed: {0}")]
    GhFailed(String),
    #[error("empty response for {0}")]
    EmptyResponse(&'static str),
    #[error("invalid JSON from GitHub for {0}: {1}")]
    BadJson(&'static str, String),
    #[error("invalid mode '{0}', expected one of test, build, release, release-notests")]
    InvalidMode(String),
    #[error("crate folder '{0}' not found at {1}")]
    CrateNotFound(String, String),
    #[error("game executable not found at {0}")]
    ExeNotFound(String),
    #[error("failed to spawn game: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("child process has no pid")]
    NoPid,
    #[error("failed to read log {0}: {1}")]
    LogRead(String, String),
    #[error("failed to create log {0}: {1}")]
    LogCreate(String, String),
    #[error("failed to kill process {0}: {1}")]
    KillFailed(u32, String),
    #[error(
        "refusing to kill pid {0}: values below 2 or above {1} parse as broadcast or process-group signals"
    )]
    InvalidPid(u32, u32),
    #[error("IO error: {0}")]
    Io(#[source] std::io::Error),
}

#[rustfmt::skip]
impl From<Error> for rmcp::ErrorData {
    fn from(value: Error) -> Self { error_method::from(value) }
}
#[cfg(test)]
mod tests {
    use crate::Error;

    #[test]
    fn test_usage() {
        let data: rmcp::ErrorData = Error::InvalidMode("nope".into()).into();
        assert!(data.message.contains("nope"));
    }
}
