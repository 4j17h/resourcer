use resourcer_core::file_io::{validate_js_path, FileAnalysisError};
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

#[tokio::test]
async fn read_js_success() {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path().with_extension("js");
    fs::write(&path, b"console.log('ok');").await.unwrap();
    let content = resourcer_core::read_js_file(&path).await.unwrap();
    assert!(content.contains("ok"));
}

#[tokio::test]
async fn read_permission_denied() {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path().with_extension("js");
    fs::write(&path, b"alert('x');").await.unwrap();
    // remove read perms
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
    let err = resourcer_core::read_js_file(&path).await.unwrap_err();
    assert!(matches!(err, FileAnalysisError::PermissionDenied(_)));
} 