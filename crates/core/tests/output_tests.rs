use std::fs;
use tempfile::tempdir;
use resourcer_core::{ensure_output_dir, mirror_structure, copy_files, validate_output};
use walkdir::WalkDir;

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

#[test]
fn mirror_structure_creates_dirs() {
    let src = tempdir().unwrap();
    // build nested dirs and files
    let nested_dir = src.path().join("d1/d2/d3");
    std::fs::create_dir_all(&nested_dir).unwrap();
    std::fs::write(src.path().join("d1/file.js"), "console.log('hi');").unwrap();

    let dst = tempdir().unwrap();
    mirror_structure(src.path(), dst.path()).expect("mirror should succeed");

    // Verify dirs exist, but file does not yet copied
    let expected_dir = dst.path().join("d1/d2/d3");
    assert!(expected_dir.exists() && expected_dir.is_dir());
    assert!(!dst.path().join("d1/file.js").exists());

    // Idempotent call
    mirror_structure(src.path(), dst.path()).expect("second call ok");
}

#[test]
fn copy_files_copies_content() {
    let src = tempdir().unwrap();
    let nested_dir = src.path().join("d1/d2");
    std::fs::create_dir_all(&nested_dir).unwrap();
    let file_a = src.path().join("d1/a.txt");
    let file_b = nested_dir.join("b.txt");
    std::fs::write(&file_a, "hello").unwrap();
    std::fs::write(&file_b, "world").unwrap();

    let dst = tempdir().unwrap();
    mirror_structure(src.path(), dst.path()).unwrap();
    copy_files(src.path(), dst.path()).unwrap();

    assert_eq!(std::fs::read_to_string(dst.path().join("d1/a.txt")).unwrap(), "hello");
    assert_eq!(std::fs::read_to_string(dst.path().join("d1/d2/b.txt")).unwrap(), "world");
}

#[test]
fn validate_output_success() {
    let src = tempdir().unwrap();
    std::fs::write(src.path().join("file.js"), "alert('x');").unwrap();

    let dst = tempdir().unwrap();
    mirror_structure(src.path(), dst.path()).unwrap();
    copy_files(src.path(), dst.path()).unwrap();

    let mismatches = validate_output(src.path(), dst.path()).unwrap();
    assert!(mismatches.is_empty());
}

#[test]
fn validate_output_detects_mismatch() {
    let src = tempdir().unwrap();
    std::fs::write(src.path().join("file.js"), "console.log('a');").unwrap();

    let dst = tempdir().unwrap();
    mirror_structure(src.path(), dst.path()).unwrap();
    copy_files(src.path(), dst.path()).unwrap();

    // Modify dst file
    std::fs::write(dst.path().join("file.js"), "modified").unwrap();

    let mismatches = validate_output(src.path(), dst.path()).unwrap();
    assert_eq!(mismatches, vec!["file.js"]);
} 