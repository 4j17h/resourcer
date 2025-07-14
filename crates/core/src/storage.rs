use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use url::Url;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Debug)]
pub struct HtmlDocument {
    pub url: Url,
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

#[async_trait::async_trait]
pub trait HtmlStorage: Send + Sync {
    async fn save_html(&self, url: Url, html: String) -> Result<(), StorageError>;
    async fn get(&self, url: &Url) -> Option<HtmlDocument>;
}

/// In-memory storage – useful for testing and small crawls.
#[derive(Default)]
pub struct MemoryStorage {
    inner: Arc<RwLock<HashMap<String, HtmlDocument>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl HtmlStorage for MemoryStorage {
    async fn save_html(&self, url: Url, html: String) -> Result<(), StorageError> {
        let doc = HtmlDocument {
            url: url.clone(),
            timestamp: Utc::now(),
            content: html,
        };
        self.inner.write().await.insert(url.as_str().to_owned(), doc);
        Ok(())
    }

    async fn get(&self, url: &Url) -> Option<HtmlDocument> {
        self.inner.read().await.get(url.as_str()).cloned()
    }
} 