use thiserror::Error;

pub type ScraperResult<T> = std::result::Result<T, ScraperError>;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("Parsing raw HTML for desired info error")]
    Parsing(String),
    #[error("Parsing raw HTML for desired info error")]
    Downloading(String),
    #[error("I/O Error")]
    IO(#[from] std::io::Error),
    #[error("Protobuf Error")]
    Protobuf(#[from] protobuf::Error),
    #[error("Request Error")]
    Request(#[from] reqwest::Error),
    #[error("Infallible error - strange")]
    Infallible(#[from] core::convert::Infallible),
}
