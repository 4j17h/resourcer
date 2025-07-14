use crate::file_io::{read_js_file, validate_js_path, FileAnalysisError};
use crate::storage::{HtmlStorage, StorageError, HtmlDocument};
use chrono::Utc;
use url::Url;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum AnalysisError {
    #[error(transparent)]
    File(#[from] FileAnalysisError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("could not convert path to file URL: {0}")]
    Url(#[from] url::ParseError),
}

/// Analyze a local JavaScript file:
/// 1. Validate and read the file.
/// 2. Store the content via provided HtmlStorage implementation.
/// Returns the stored HtmlDocument instance.
pub async fn analyze_local_js<S>(path: &Path, store: &S) -> Result<HtmlDocument, AnalysisError>
where
    S: HtmlStorage,
{
    let canonical = validate_js_path(path).await?;
    let content = read_js_file(&canonical).await?;

    let file_url = Url::from_file_path(&canonical).map_err(|_| url::ParseError::IdnaError)?;
    store.save_html(file_url.clone(), content.clone()).await?;

    Ok(HtmlDocument {
        url: file_url,
        timestamp: Utc::now(),
        content,
    })
}

pub async fn analyze_local_js_with_sourcemaps<S>(path: &Path, store: &S) -> Result<(HtmlDocument, Vec<Url>), AnalysisError>
where
    S: HtmlStorage,
{
    let doc = analyze_local_js(path, store).await?;
    // Re-use the stored document’s file URL as the base for relative sourcemap resolution.
    let maps = crate::sourcemap::find_sourcemap_urls(&doc.url, &doc.content);
    Ok((doc, maps))
}
