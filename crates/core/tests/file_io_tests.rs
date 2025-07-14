use sourcedumper_core::file_io::{validate_js_path, FileAnalysisError};
use tempfile::NamedTempFile;
use tokio::fs;
use std::path::PathBuf;

#[tokio::test]
async fn valid_js_file() {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path().with_extension("js");
    fs::write(&path, b"console.log('hi');").await.unwrap();

    let validated = validate_js_path(&path).await.unwrap();
    assert_eq!(validated, std::fs::canonicalize(&path).unwrap());
}

#[tokio::test]
async fn missing_file() {
    let missing = PathBuf::from("/tmp/nonexistent.js");
    let err = validate_js_path(&missing).await.unwrap_err();
    assert!(matches!(err, FileAnalysisError::NotFound(_)));
}

#[tokio::test]
async fn wrong_extension() {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path().with_extension("txt");
    fs::write(&path, b"hello").await.unwrap();
    let err = validate_js_path(&path).await.unwrap_err();
    assert!(matches!(err, FileAnalysisError::InvalidExtension(_)));
} 