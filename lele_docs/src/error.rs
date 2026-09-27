use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("lele_lint error: {0}")]
    Lint(#[from] lele_lint::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("server error: {0}")]
    Server(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bad request: {0}")]
    BadRequest(String),
}

// no test_usage necessary
