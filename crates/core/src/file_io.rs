use std::path::{Path, PathBuf};

use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum FileAnalysisError {
    #[error("file not found: {0}")]
    NotFound(PathBuf),
    #[error("invalid extension (expected .js): {0}")]
    InvalidExtension(PathBuf),
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Validate that the given path exists, is a regular `.js` file, and return a canonicalized absolute path.
/// Errors are mapped to `FileAnalysisError` variants.
pub async fn validate_js_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, FileAnalysisError> {
    let path_ref = path.as_ref();

    // Check extension
    if path_ref.extension().and_then(|e| e.to_str()).map_or(true, |ext| ext != "js") {
        return Err(FileAnalysisError::InvalidExtension(path_ref.to_path_buf()));
    }

    // Metadata fetch (async)
    match fs::metadata(path_ref).await {
        Ok(meta) => {
            if !meta.is_file() {
                return Err(FileAnalysisError::InvalidExtension(path_ref.to_path_buf()));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(FileAnalysisError::NotFound(path_ref.to_path_buf()));
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(FileAnalysisError::PermissionDenied(path_ref.to_path_buf()));
        }
        Err(e) => return Err(FileAnalysisError::Io(e)),
    }

    // Canonicalize to absolute path (blocking; use tokio::task::spawn_blocking to avoid blocking reactor)
    let canonical = tokio::task::spawn_blocking(move || std::fs::canonicalize(path_ref))
        .await
        .map_err(|e| FileAnalysisError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))??;

    Ok(canonical)
} 