pub mod fetch;

pub use fetch::{fetch_html, FetchError};
pub mod storage;
pub use storage::{HtmlStorage, MemoryStorage, StorageError, HtmlDocument};
