use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use reqwest::Client;

use crate::fetch::{fetch_with_retries, FetchError};

/// Result of a download attempt
pub struct DownloadResult {
    pub url: String,
    pub content: Option<String>,
    pub error: Option<FetchError>,
}

/// Download manager configuration
pub struct DownloadManagerConfig {
    pub concurrency: usize,
    pub retry_attempts: usize,
}

impl Default for DownloadManagerConfig {
    fn default() -> Self {
        Self {
            concurrency: 8,
            retry_attempts: 3,
        }
    }
}

/// Concurrently download a list of URLs using a worker pool.
pub async fn download_many(urls: Vec<String>, config: DownloadManagerConfig) -> Vec<DownloadResult> {
    let (tx, rx) = mpsc::channel::<String>(config.concurrency * 2);
    let (result_tx, mut result_rx) = mpsc::channel::<DownloadResult>(config.concurrency * 2);
    let client = Arc::new(Client::builder().build().unwrap());
    let rx = Arc::new(Mutex::new(rx));

    // Spawn workers
    let mut handles = Vec::new();
    for _ in 0..config.concurrency {
        let rx = Arc::clone(&rx);
        let result_tx = result_tx.clone();
        let client = client.clone();
        let attempts = config.retry_attempts;
        let handle = task::spawn(async move {
            loop {
                let url = {
                    let mut rx = rx.lock().await;
                    rx.recv().await
                };
                let url = match url {
                    Some(u) => u,
                    None => break,
                };
                let res = fetch_with_retries(&url, attempts).await;
                let (content, error) = match res {
                    Ok(body) => (Some(body), None),
                    Err(e) => (None, Some(e)),
                };
                let _ = result_tx.send(DownloadResult { url: url.clone(), content, error }).await;
            }
        });
        handles.push(handle);
    }

    // Feed URLs to workers
    for url in urls {
        let _ = tx.send(url).await;
    }
    drop(tx); // Close channel
    drop(result_tx);

    // Wait for all workers to finish
    for handle in handles {
        let _ = handle.await;
    }

    // Collect results
    let mut results = Vec::new();
    while let Some(res) = result_rx.recv().await {
        results.push(res);
    }
    results
} 