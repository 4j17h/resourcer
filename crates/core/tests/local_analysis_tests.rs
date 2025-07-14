use sourcedumper_core::{local_analysis::analyze_local_js, storage::MemoryStorage};
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