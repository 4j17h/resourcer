use resourcer_core::{parse_sourcemap, sources_list};

#[test]
fn parse_basic_sourcemap() {
    let json = r#"{
        "version":3,
        "file":"out.js",
        "sourceRoot":"",
        "sources":["foo.ts"],
        "names":[],
        "mappings":"AAAA"
    }"#;
    let sm = parse_sourcemap(json).unwrap();
    let sources = sources_list(&sm);
    assert_eq!(sources, vec!["foo.ts"]);
} 