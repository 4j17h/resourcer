use std::fs;
use tempfile::tempdir;
use sourcedumper_core::ensure_output_dir;

#[test]
fn creates_missing_directory() {
    let dir = tempdir().unwrap();
    let nested = dir.path().join("a/b/c");
    ensure_output_dir(&nested).expect("should create dirs");
    assert!(nested.exists() && nested.is_dir());
}

#[test]
fn ok_if_directory_exists() {
    let dir = tempdir().unwrap();
    ensure_output_dir(dir.path()).expect("existing dir should be ok");
}

#[test]
fn error_if_path_is_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "data").unwrap();
    let err = ensure_output_dir(&file_path).expect_err("should error when path is a file");
    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
} 