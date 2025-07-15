use resourcer_core::{parse_sourcemap, reconstruct_paths};
use tempfile::tempdir;

#[test]
fn webpack_protocol_and_source_root() {
    let json = r#"{
        "version":3,
        "sourceRoot":"src/",
        "sources":["webpack:///./foo/bar.js","webpack://namespace/./baz.js"]
    }"#;
    let sm = parse_sourcemap(json).unwrap();
    let dir = tempdir().unwrap();

    let paths = reconstruct_paths(dir.path(), &sm);
    let expected1 = dir.path().join("src/foo/bar.js");
    let expected2 = dir.path().join("src/baz.js");

    assert_eq!(paths, vec![expected1, expected2]);
} 