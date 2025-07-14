use reqwest::{Client, Url};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("invalid url: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("unsupported url scheme: {0}")]
    UnsupportedScheme(String),
    #[error(transparent)]
    Network(#[from] reqwest::Error),
}

/// Fetch HTML content from the given URL and return it as a UTF-8 string.
/// A desktop-like User-Agent header is added and request times out after 30 seconds.
pub async fn fetch_html(url: &str) -> Result<String, FetchError> {
    let url = Url::parse(url)?;
    match url.scheme() {
        "http" | "https" => {},
        other => return Err(FetchError::UnsupportedScheme(other.to_string())),
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0 Safari/537.36")
        .build()?;

    let html = client
        .get(url)
        .send()
        .await?
        .error_for_status()? // convert non-2xx into error
        .text()
        .await?;

    Ok(html)
} 