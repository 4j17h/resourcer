use sourcedumper_core::extract_sourcemap_urls;

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