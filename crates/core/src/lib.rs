pub mod fetch;

pub use fetch::{fetch_html, FetchError};
pub mod storage;
pub use storage::{HtmlStorage, MemoryStorage, StorageError, HtmlDocument};
pub mod file_io;

pub use file_io::{validate_js_path, read_js_file, FileAnalysisError};

pub mod local_analysis;

pub use local_analysis::{analyze_local_js, AnalysisError};

pub mod sourcemap;
pub use sourcemap::{extract_sourcemap_urls, validate_sourcemap_urls};
