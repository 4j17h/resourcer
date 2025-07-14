use sourcedumper_core::{local_analysis::analyze_local_js, storage::MemoryStorage};
use sourcedumper_core::HtmlStorage;
use tempfile::NamedTempFile;
use tokio::fs;

#[tokio::test]
async fn analyze_and_store() {
    let store = MemoryStorage::new();

    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path().with_extension("js");
    fs::write(&path, b"console.log('stored');").await.unwrap();

    let doc = analyze_local_js(&path, &store).await.unwrap();
    let retrieved = store.get(&doc.url).await.unwrap();
    assert_eq!(retrieved.content, "console.log('stored');");
}

#[tokio::test]
async fn analyze_and_detect_sourcemaps() {
    let store = MemoryStorage::new();

    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path().with_extension("js");
    tokio::fs::write(&path, b"console.log('x');\n//# sourceMappingURL=map1.js.map").await.unwrap();

    let (_doc, maps) = sourcedumper_core::local_analysis::analyze_local_js_with_sourcemaps(&path, &store).await.unwrap();
    assert_eq!(maps.len(), 1);
    assert!(maps[0].as_str().ends_with("map1.js.map"));
} 