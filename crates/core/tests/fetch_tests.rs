use sourcedumper_core::fetch::fetch_html;
use httpmock::prelude::*;

#[tokio::test]
async fn fetch_html_success() {
    let server = MockServer::start_async().await;
    let m = server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(200)
            .header("Content-Type", "text/html")
            .body("<html>hello</html>");
    });

    let url = format!("{}", server.base_url());
    let body = fetch_html(&url).await.unwrap();
    m.assert();
    assert!(body.contains("hello"));
}

#[tokio::test]
async fn fetch_html_http_error() {
    let server = MockServer::start_async().await;
    server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(404);
    });
    let url = server.base_url();
    let err = fetch_html(&url).await.unwrap_err();
    assert!(matches!(err, sourcedumper_core::FetchError::HttpStatus(404)));
}

#[tokio::test]
async fn invalid_scheme() {
    let err = fetch_html("ftp://example.com").await.unwrap_err();
    assert!(matches!(err, sourcedumper_core::FetchError::UnsupportedScheme(_)));
} 