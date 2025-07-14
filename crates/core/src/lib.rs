pub mod fetch;

pub use fetch::{fetch_html, FetchError};
pub mod storage;
pub use storage::{HtmlStorage, MemoryStorage, StorageError, HtmlDocument};
pub mod file_io;

pub use file_io::{validate_js_path, read_js_file, FileAnalysisError};
