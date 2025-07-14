use sourcedumper_core::{infer_chunk_filename_template, ChunkFilenameTemplate, extract_public_path, build_chunk_url, validate_chunk_urls};
use url::Url;
use httpmock::MockServer;
use httpmock::Method::HEAD;

#[test]
fn infer_standard_template() {
    let runtime = r#"
    __webpack_require__.u = function(chunkId) {
        return "static/js/" + chunkId + ".js";
    };
    "#;

    let tpl = infer_chunk_filename_template(runtime).expect("should detect template");
    assert_eq!(tpl, ChunkFilenameTemplate { prefix: "static/js/".into(), suffix: ".js".into() });
}

#[test]
fn no_template_found() {
    let runtime = "console.log('no webpack');";
    assert!(infer_chunk_filename_template(runtime).is_none());
}

#[test]
fn extract_public_and_build_url() {
    let js = r#"__webpack_require__.p = "https://cdn.example.com/assets/";"#;
    let public = extract_public_path(js).expect("public path");
    assert_eq!(public, "https://cdn.example.com/assets/");

    let tpl = ChunkFilenameTemplate { prefix: "js/".into(), suffix: ".js".into() };
    let url = build_chunk_url(Url::parse(&public).ok().as_ref(), &tpl, "123").expect("url");
    assert_eq!(url.as_str(), "https://cdn.example.com/assets/js/123.js");
}

#[test]
fn infer_arrow_template() {
    let runtime = r#"__webpack_require__.u = (id) => "chunks/" + id + ".chunk.js";"#;
    let tpl = infer_chunk_filename_template(runtime).expect("detect");
    assert_eq!(tpl, ChunkFilenameTemplate { prefix: "chunks/".into(), suffix: ".chunk.js".into() });
}

#[test]
fn infer_template_literal() {
    let runtime = r#"__webpack_require__.u = function(id) { return `assets/${id}.bundle.js`; };"#;
    let tpl = infer_chunk_filename_template(runtime).expect("detect");
    assert_eq!(tpl, ChunkFilenameTemplate { prefix: "assets/".into(), suffix: ".bundle.js".into() });
}

#[tokio::test]
async fn validate_urls() {
    let server = MockServer::start_async().await;
    let ok_mock = server.mock_async(|when, then| {
        when.method(HEAD).path("/chunk.js");
        then.status(200);
    }).await;

    let bad_mock = server.mock_async(|when, then| {
        when.method(HEAD).path("/bad.js");
        then.status(404);
    }).await;

    let ok_url = Url::parse(&format!("{}/chunk.js", server.base_url())).unwrap();
    let bad_url = Url::parse(&format!("{}/bad.js", server.base_url())).unwrap();

    let res = validate_chunk_urls(vec![ok_url.clone(), bad_url]).await;
    assert_eq!(res, vec![ok_url]);

    ok_mock.assert_async().await;
}

#[test]
fn swc_parse_runtime_prints_ast() {
    let js = r#"
    __webpack_require__.u = function(chunkId) {
        return "static/js/" + chunkId + ".js";
    };
    var foo = 42;
    "#;
    sourcedumper_core::swc_print_top_level(js);
} 