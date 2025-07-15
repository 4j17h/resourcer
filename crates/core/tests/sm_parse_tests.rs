use std::fs;
use tempfile::tempdir;
use sourcemap::SourceMap;
use resourcer_core::sm_parse::{parse_sourcemap, reconstruct_sources_with_swc};

#[test]
fn test_reconstruct_with_sources_content() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("foo.js");
    let sm_json = format!(r#"{{
        "version":3,
        "file":"out.js",
        "sources":["{}"],
        "sourcesContent":["console.log('hi');\n"],
        "mappings":";"
    }}"#, file_path.display().to_string().replace('\\', "\\\\"));
    let sm = parse_sourcemap(&sm_json).unwrap();
    reconstruct_sources_with_swc(&sm, "").unwrap();
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "console.log('hi');\n");
}

#[test]
fn test_reconstruct_with_mapping_stub() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("bar.js");
    let sm_json = format!(r#"{{
        "version":3,
        "file":"out.js",
        "sources":["{}"],
        "mappings":"AAAA;",
        "names":[],
        "sourcesContent":null
    }}"#, file_path.display().to_string().replace('\\', "\\\\"));
    let sm = parse_sourcemap(&sm_json).unwrap();
    // Minimal generated JS
    let generated_js = "var x = 1;\n";
    reconstruct_sources_with_swc(&sm, generated_js).unwrap();
    let content = fs::read_to_string(&file_path).unwrap();
    // The content will be a stub/placeholder or segment, just check file is written
    assert!(!content.is_empty());
} 