use sourcemap::SourceMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourcemapError {
    #[error("invalid JSON or sourcemap: {0}")]
    Parse(String),
}

/// Parse a sourcemap JSON string and return the `sourcemap::SourceMap` object.
pub fn parse_sourcemap(json: &str) -> Result<SourceMap, SourcemapError> {
    SourceMap::from_slice(json.as_bytes()).map_err(|e| SourcemapError::Parse(e.to_string()))
}

/// Convenience helper: return the list of original source paths contained in the map.
pub fn sources_list(sm: &SourceMap) -> Vec<String> {
    (0..sm.get_source_count())
        .filter_map(|i| sm.get_source(i).map(|s| s.to_string()))
        .collect()
} 