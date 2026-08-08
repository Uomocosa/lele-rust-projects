use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("spawn failed: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("child stdio pipe unavailable")]
    PipeUnavailable,
    #[error("unknown pid {0}")]
    UnknownPid(u32),
    #[error("stdin channel closed")]
    StdinClosed,
    #[error("screenshot failed: {0}")]
    Screenshot(String),
    #[error("telegram error: {0}")]
    Telegram(String),
    #[error("window error: {0}")]
    Window(String),
}

#[rustfmt::skip]
impl From<Error> for rmcp::ErrorData {
    fn from(value: Error) -> Self { crate::ErrorMethod::from(value) }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    #[test]
    fn test_usage() {
        let data: rmcp::ErrorData = Error::UnknownPid(7).into();
        assert!(data.message.contains("7"));
    }
}
