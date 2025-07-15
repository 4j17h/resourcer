use regex::Regex;
use url::Url;

/// Find sourceMappingURL comment in JavaScript content
pub fn find_sourcemap_url_in_js(js: &str) -> Option<String> {
    let re = Regex::new(r#"(?://|/\*)# sourceMappingURL=([^\s*]+)"#).unwrap();
    re.captures(js).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Derive a base URL for chunk files using the runtime URL and the detected prefix.
/// If the runtime URL's path contains the prefix (e.g. "/_next/static/chunks/") we
/// strip everything from the prefix onward so that `base + prefix` exactly
/// reproduces the runtime's directory structure. Otherwise we fall back to the
/// page origin.
pub fn derive_base_from_runtime(runtime_url: &Url, prefix: &str) -> Url {
    // Build origin (scheme + host) first
    let mut origin = Url::parse(&format!("{}://{}", runtime_url.scheme(), runtime_url.host_str().unwrap_or(""))).unwrap();
    // Try to locate the prefix inside the runtime's path
    let path = runtime_url.path();
    if let Some(idx) = path.rfind(prefix) {
        let base_path = &path[..idx]; // up to but excluding prefix
        let mut new_path = base_path.to_string();
        if !new_path.ends_with('/') {
            new_path.push('/');
        }
        origin.set_path(&new_path);
        origin
    } else {
        origin.set_path("/");
        origin
    }
}

/// Extract script URLs from HTML content
pub fn extract_script_urls(html: &str, base: &Url) -> Vec<Url> {
    let script_re: Regex = Regex::new(r#"<script[^>]*?src=["']([^"']+?\.js[^"']*)["']"#).unwrap();
    let mut script_urls = Vec::new();
    for caps in script_re.captures_iter(html) {
        if let Some(src) = caps.get(1) {
            if let Ok(joined) = base.join(src.as_str()) {
                script_urls.push(joined);
            }
        }
    }
    script_urls
} 