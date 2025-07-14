use std::path::{Path, PathBuf};
use sourcemap::SourceMap;

/// Reconstruct canonical file paths for each `sources` entry in a sourcemap.
///
/// * `base_dir` – root directory where reconstructed files should live (used for
///   absolute/relative resolution).
///
/// Rules:
/// 1. If a `sourceRoot` is present, prepend it to relative sources.
/// 2. Strip `webpack://` protocol and optional namespace (e.g., `webpack:///` or `webpack://src/`).
/// 3. Normalize `..`, leading `./`, and convert URL separators to the platform’s path separator.
/// 4. Return Vec<PathBuf> in the same order as `sm.get_source_count()`.
pub fn reconstruct_paths(base_dir: &Path, sm: &SourceMap) -> Vec<PathBuf> {
    let root = sm.get_source_root().unwrap_or("");

    (0..sm.get_source_count())
        .filter_map(|i| sm.get_source(i))
        .map(|src| {
            let mut s = src.to_string();

            // Remove sourceRoot prefix if present
            if !root.is_empty() && s.starts_with(root) {
                s = s[root.len()..].to_string();
            }

            if let Some(rest) = s.strip_prefix("webpack://") {
                // After "webpack://", find the first '/' and take everything after it.
                // e.g., webpack:///./foo/bar.js -> /./foo/bar.js -> find '/' -> ./foo/bar.js
                // e.g., webpack://namespace/./baz.js -> /namespace/./baz.js -> find '/' -> ./baz.js
                if let Some(slash_index) = rest.find('/') {
                    s = rest[slash_index + 1..].to_string();
                } else {
                     // No slash found after webpack://, take the entire rest
                     s = rest.to_string();
                }
            }
            s = s.trim_start_matches("./").to_string();

            // Prepend sourceRoot if present and the source path is relative
            let combined = if !root.is_empty() && !Path::new(&s).is_absolute() {
                // Ensure root has a trailing slash for correct path joining
                let formatted_root = if root.ends_with('/') {
                    root.to_string()
                } else {
                    format!("{}/", root)
                };
                format!("{}{}", formatted_root, s)
            } else {
                s
            };

            let p = PathBuf::from(combined);
            base_dir.join(p)
        })
        .collect()
} 