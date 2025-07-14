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
    #[error("http status {0}")]
    HttpStatus(u16),
    #[error("request timed out")]
    Timeout,
}

/// Validate and parse a URL string, returning Url or FetchError
pub fn validate_url(input: &str) -> Result<Url, FetchError> {
    let url = Url::parse(input)?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        other => Err(FetchError::UnsupportedScheme(other.into())),
    }
}

/// Fetch with limited retry attempts on network/timeouts using exponential backoff
pub async fn fetch_with_retries(url: &str, attempts: usize) -> Result<String, FetchError> {
    let mut delay = Duration::from_millis(200);
    let url_owned = url.to_string();
    let mut last_err: Option<FetchError> = None;
    for _ in 0..attempts {
        match fetch_html(&url_owned).await {
            Ok(body) => return Ok(body),
            Err(e @ FetchError::Timeout) | Err(e @ FetchError::Network(_)) => {
                last_err = Some(e);
                tokio::time::sleep(delay).await;
                delay *= 2;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or(FetchError::Timeout))
}

/// Fetch HTML content from the given URL and return it as a UTF-8 string.
/// A desktop-like User-Agent header is added and request times out after 30 seconds.
pub async fn fetch_html(url: &str) -> Result<String, FetchError> {
    let url = validate_url(url)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0 Safari/537.36")
        .build()?;

    let resp = client.get(url).send().await.map_err(|e| {
        if e.is_timeout() {
            FetchError::Timeout
        } else {
            FetchError::Network(e)
        }
    })?;

    let status = resp.status();
    if !status.is_success() {
        return Err(FetchError::HttpStatus(status.as_u16()));
    }

    Ok(resp.text().await?)
} 