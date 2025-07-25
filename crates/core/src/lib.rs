pub mod fetch;

pub use fetch::{fetch_html, FetchError};
pub mod storage;
pub use storage::{HtmlStorage, MemoryStorage, StorageError, HtmlDocument};
pub mod file_io;

pub use file_io::{validate_js_path, read_js_file, FileAnalysisError};

pub mod local_analysis;

pub use local_analysis::{analyze_local_js, AnalysisError};

pub mod sourcemap;
pub use sourcemap::{extract_sourcemap_urls, validate_sourcemap_urls, find_sourcemap_urls};

pub mod webpack;
pub use webpack::{infer_chunk_filename_template, ChunkFilenameTemplate, extract_public_path, build_chunk_url, validate_chunk_urls, swc_print_top_level, extract_chunk_ids, generate_chunk_urls, extract_literal_chunk_paths, extract_chunk_maps, generate_urls_from_chunk_maps, extract_paths_from_build_manifest};

pub mod sm_parse;
pub use sm_parse::{parse_sourcemap, SourcemapError, sources_list};

pub mod path_reconstruct;
pub use path_reconstruct::reconstruct_paths;

pub mod download_manager;
pub use download_manager::{download_many, DownloadManagerConfig, DownloadResult};

pub mod output;
pub use output::{ensure_output_dir, mirror_structure, copy_files, validate_output};

pub mod url_utils;
pub use url_utils::{find_sourcemap_url_in_js, derive_base_from_runtime, extract_script_urls};

pub mod cli_ops;
pub use cli_ops::{save_js_and_sources, handle_list_urls, CLIError};
