use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("config error: {0}")]
    Config(String),
    #[error("discord error: {0}")]
    Discord(Box<serenity::Error>),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("scrape error: {0}")]
    Scrape(String),
}

impl From<serenity::Error> for Error {
    fn from(e: serenity::Error) -> Self {
        Error::Discord(Box::new(e))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
