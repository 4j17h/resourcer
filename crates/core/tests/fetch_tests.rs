use core::fetch::fetch_html;
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