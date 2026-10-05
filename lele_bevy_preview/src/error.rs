use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("preview directory could not be prepared: {0}")]
    Io(String),

    #[error("preview scene is invalid: {0}")]
    Scene(String),

    #[error("no frame was captured within {0} updates")]
    NoFrame(u32),

    #[error("captured frame is empty: {0}")]
    BlankFrame(String),

    #[error(
        "captured frame at {path} has only {distinct} distinct colour(s); expected at least {min} (nothing rendered?)"
    )]
    NoVisual {
        path: String,
        distinct: usize,
        min: usize,
    },

    #[error("ffmpeg is required to build an mp4 preview but was not found on PATH")]
    NoFfmpeg,

    #[error("ffmpeg failed for {path}: {message}")]
    Ffmpeg { path: String, message: String },

    #[error("telegram delivery failed: {0}")]
    Telegram(String),
}
// no test_usage necessary
