use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;
use reqwest::Client;
use futures::stream::{FuturesUnordered, StreamExt};

/// Represents the discovered pattern that Webpack uses to construct chunk URLs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkFilenameTemplate {
    pub prefix: String,
    pub suffix: String,
}

// Regexes for various helper forms; tried in order until one matches.
static CHUNK_URL_FN_RE: Lazy<Regex> = Lazy::new(|| {
    // Classic anonymous function form
    Regex::new(r#"__webpack_require__\.u\s*=\s*function[^\{]*\{[^}]*?return\s+"([^"]*)"\s*\+\s*[^+]+\+\s*"([^"]*)";"#).unwrap()
});

static CHUNK_URL_ARROW_RE: Lazy<Regex> = Lazy::new(|| {
    // Arrow function: __webpack_require__.u = (chunkId) => "prefix" + chunkId + "suffix";
    Regex::new(r#"__webpack_require__\.u\s*=\s*\([^)]*\)\s*=>\s*"([^"]*)"\s*\+\s*[^+]+\+\s*"([^"]*)";"#).unwrap()
});

static CHUNK_URL_TMPL_RE: Lazy<Regex> = Lazy::new(|| {
    // Template literal form: return `prefix${chunkId}suffix`;
    Regex::new(r#"__webpack_require__\.u\s*=\s*function[^\{]*\{[^}]*?return\s+`([^`]*?)\$\{[^}]+}([^`]*?)`;"#).unwrap()
});

// Pattern for public path assignment.
static PUBLIC_PATH_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"__webpack_require__\.p\s*=\s*"([^"]*)";"#).unwrap()
});

/// Infer prefix & suffix used to build chunk URLs.
pub fn infer_chunk_filename_template(js: &str) -> Option<ChunkFilenameTemplate> {
    for re in [&*CHUNK_URL_FN_RE, &*CHUNK_URL_ARROW_RE, &*CHUNK_URL_TMPL_RE] {
        if let Some(caps) = re.captures(js) {
            return Some(ChunkFilenameTemplate {
                prefix: caps.get(1).unwrap().as_str().to_string(),
                suffix: caps.get(2).unwrap().as_str().to_string(),
            });
        }
    }
    None
}

/// Extract configured public path, if present.
pub fn extract_public_path(js: &str) -> Option<String> {
    let caps = PUBLIC_PATH_RE.captures(js)?;
    let path = caps.get(1)?.as_str();
    if path.is_empty() { None } else { Some(path.to_string()) }
}

/// Build a full chunk URL from components.
pub fn build_chunk_url(base: Option<&Url>, template: &ChunkFilenameTemplate, chunk_id: &str) -> Option<Url> {
    let path = format!("{}{}{}", template.prefix, chunk_id, template.suffix);
    if let Some(base_url) = base {
        base_url.join(&path).ok()
    } else {
        Url::parse(&path).ok()
    }
}

/// Perform concurrent HTTP HEAD requests (falling back to GET if HEAD not allowed) to verify that
/// discovered chunk URLs are reachable. Returns the subset of URLs that responded with a 2xx
/// status code.
pub async fn validate_chunk_urls(urls: impl IntoIterator<Item = Url>) -> Vec<Url> {
    let client = Client::builder().redirect(reqwest::redirect::Policy::limited(5)).build().unwrap();
    let mut futs = FuturesUnordered::new();
    for url in urls {
        let c = client.clone();
        futs.push(async move {
            match c.head(url.clone()).send().await {
                Ok(resp) if resp.status().is_success() => Some(url.clone()),
                Ok(resp) if resp.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED => {
                    // fallback to GET
                    match c.get(url.clone()).send().await {
                        Ok(r) if r.status().is_success() => Some(url.clone()),
                        _ => None,
                    }
                }
                _ => None,
            }
        });
    }
    let mut good = Vec::new();
    while let Some(res) = futs.next().await {
        if let Some(u) = res { good.push(u) }
    }
    good
} 