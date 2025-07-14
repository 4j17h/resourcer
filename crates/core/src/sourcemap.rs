use regex::Regex;
use once_cell::sync::Lazy;

static SOURCE_MAP_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)//[#@]\s*sourceMappingURL=([^\s]+)|/\*#\s*sourceMappingURL=([^*]+)\*/").unwrap()
});

/// Extract raw sourcemap URLs from JavaScript text.
/// Returns a vector preserving original order.
pub fn extract_sourcemap_urls(js: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for caps in SOURCE_MAP_RE.captures_iter(js) {
        if let Some(m) = caps.get(1).or_else(|| caps.get(2)) {
            urls.push(m.as_str().trim().to_string());
        }
    }
    urls
} 