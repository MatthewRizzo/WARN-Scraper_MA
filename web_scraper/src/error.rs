use thiserror::Error;

pub type ScraperResult<T> = std::result::Result<T, ScraperError>;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("Parsing raw HTML for desired info error")]
    Parsing(String),
    #[error("Parsing the xlsx file resulted in an error")]
    ExcelParsing(String),
    #[error("Downloading historical / yearly data")]
    Downloading(String),
    #[error("Merging Warn Notices")]
    MergingNotices(String),
    #[error("I/O Error")]
    IO(#[from] std::io::Error),
    #[error("Protobuf Error")]
    Protobuf(#[from] protobuf::Error),
    #[error("Request Error")]
    Request(#[from] reqwest::Error),
    #[error("Infallible error - strange")]
    Infallible(#[from] core::convert::Infallible),
    // #[error("Excel Error")]
    // Excel(#[from] office::Error),
}
