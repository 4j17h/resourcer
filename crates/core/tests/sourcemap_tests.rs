use resourcer_core::extract_sourcemap_urls;
use url::Url;

#[test]
fn single_line_comment() {
    let js = "console.log(1);\n//# sourceMappingURL=app.js.map\n";
    let urls = extract_sourcemap_urls(js);
    assert_eq!(urls, vec!["app.js.map"]);
}

#[test]
fn block_comment() {
    let js = "/*# sourceMappingURL=vendor.map */\nfunction x(){}";
    let urls = extract_sourcemap_urls(js);
    assert_eq!(urls, vec!["vendor.map"]);
}

#[test]
fn multiple_and_none() {
    let js = "//# sourceMappingURL=first.map\n/*# sourceMappingURL=second.map */\nconsole.log('x');";
    let urls = extract_sourcemap_urls(js);
    assert_eq!(urls, vec!["first.map", "second.map"]);

    let none = "function test() {}";
    assert!(extract_sourcemap_urls(none).is_empty());
}

#[test]
fn validate_and_dedup() {
    let base = Url::parse("https://example.com/js/app.js").unwrap();
    let raw = vec!["app.js.map".to_string(), "https://cdn.com/vendor.map".to_string(), "app.js.map".to_string()];
    let urls = resourcer_core::validate_sourcemap_urls(&base, raw);
    assert_eq!(urls.len(), 2);
    assert!(urls.iter().any(|u| u.as_str() == "https://example.com/js/app.js.map"));
    assert!(urls.iter().any(|u| u.as_str() == "https://cdn.com/vendor.map"));
} 