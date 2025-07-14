use sourcedumper_core::fetch::fetch_html;
use sourcedumper_core::download_manager::{download_many, DownloadManagerConfig};
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

#[tokio::test]
async fn concurrent_download_many() {
    let server = MockServer::start_async().await;
    let ok1 = server.mock(|when, then| {
        when.method(GET).path("/a");
        then.status(200).body("A");
    });
    let ok2 = server.mock(|when, then| {
        when.method(GET).path("/b");
        then.status(200).body("B");
    });
    let fail = server.mock(|when, then| {
        when.method(GET).path("/fail");
        then.status(500);
    });
    let urls = vec![
        format!("{}/a", server.base_url()),
        format!("{}/b", server.base_url()),
        format!("{}/fail", server.base_url()),
        "http://invalid-url".to_string(),
    ];
    let config = DownloadManagerConfig { concurrency: 2, retry_attempts: 2 };
    let results = download_many(urls, config).await;
    assert_eq!(results.len(), 4);
    let mut ok_count = 0;
    let mut fail_count = 0;
    for r in results {
        match (r.content, r.error) {
            (Some(body), None) => {
                assert!(body == "A" || body == "B");
                ok_count += 1;
            }
            (None, Some(_)) => {
                fail_count += 1;
            }
            _ => panic!("Unexpected result variant"),
        }
    }
    assert_eq!(ok_count, 2);
    assert_eq!(fail_count, 2);
} 