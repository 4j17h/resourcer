use resourcer_core::storage::{HtmlStorage, MemoryStorage};
use url::Url;

#[tokio::test]
async fn save_and_get_round_trip() {
    let store = MemoryStorage::new();
    let url = Url::parse("https://example.com").unwrap();
    store
        .save_html(url.clone(), "<html>hello</html>".to_string())
        .await
        .unwrap();
    let doc = store.get(&url).await.unwrap();
    assert_eq!(doc.url, url);
    assert!(doc.content.contains("hello"));
} 