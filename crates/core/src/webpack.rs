use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;
use reqwest::Client;
use futures::stream::{FuturesUnordered, StreamExt};
use swc_ecma_parser::{Parser, StringInput, Syntax};
use swc_common::{sync::Lrc, SourceMap, FileName};
use std::collections::HashSet;

/// Represents the discovered pattern that Webpack uses to construct chunk URLs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkFilenameTemplate {
    pub prefix: String,
    pub suffix: String,
}

// Regexes for various helper forms; tried in order until one matches.
static CHUNK_URL_FN_RE: Lazy<Regex> = Lazy::new(|| {
    // Classic anonymous function form
    Regex::new(r#"__webpack_require__\.u\s*=\s*function[^\{]*\{[^}]*?return\s+"([^"]*)"\s*\+\s*[^+]+\+\s*"([^"]*)";"#).unwrap()
});

static CHUNK_URL_ARROW_RE: Lazy<Regex> = Lazy::new(|| {
    // Arrow function: __webpack_require__.u = (chunkId) => "prefix" + chunkId + "suffix";
    Regex::new(r#"__webpack_require__\.u\s*=\s*\([^)]*\)\s*=>\s*"([^"]*)"\s*\+\s*[^+]+\+\s*"([^"]*)";"#).unwrap()
});

static CHUNK_URL_TMPL_RE: Lazy<Regex> = Lazy::new(|| {
    // Template literal form: return `prefix${chunkId}suffix`;
    Regex::new(r#"__webpack_require__\.u\s*=\s*function[^\{]*\{[^}]*?return\s+`([^`]*?)\$\{[^}]+}([^`]*?)`;"#).unwrap()
});

// Pattern for public path assignment.
static PUBLIC_PATH_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"__webpack_require__\.p\s*=\s*"([^"]*)";"#).unwrap()
});

/// Infer prefix & suffix used to build chunk URLs.
pub fn infer_chunk_filename_template(js: &str) -> Option<ChunkFilenameTemplate> {
    for re in [&*CHUNK_URL_FN_RE, &*CHUNK_URL_ARROW_RE, &*CHUNK_URL_TMPL_RE] {
        if let Some(caps) = re.captures(js) {
            return Some(ChunkFilenameTemplate {
                prefix: caps.get(1).unwrap().as_str().to_string(),
                suffix: caps.get(2).unwrap().as_str().to_string(),
            });
        }
    }
    None
}

/// Extract configured public path, if present.
pub fn extract_public_path(js: &str) -> Option<String> {
    let caps = PUBLIC_PATH_RE.captures(js)?;
    let path = caps.get(1)?.as_str();
    if path.is_empty() { None } else { Some(path.to_string()) }
}

/// Build a full chunk URL from components.
pub fn build_chunk_url(base: Option<&Url>, template: &ChunkFilenameTemplate, chunk_id: &str) -> Option<Url> {
    let path = format!("{}{}{}", template.prefix, chunk_id, template.suffix);
    if let Some(base_url) = base {
        base_url.join(&path).ok()
    } else {
        Url::parse(&path).ok()
    }
}

/// Perform concurrent HTTP HEAD requests (falling back to GET if HEAD not allowed) to verify that
/// discovered chunk URLs are reachable. Returns the subset of URLs that responded with a 2xx
/// status code.
pub async fn validate_chunk_urls(urls: impl IntoIterator<Item = Url>) -> Vec<Url> {
    let client = Client::builder().redirect(reqwest::redirect::Policy::limited(5)).build().unwrap();
    let mut futs = FuturesUnordered::new();
    for url in urls {
        let c = client.clone();
        futs.push(async move {
            match c.head(url.clone()).send().await {
                Ok(resp) if resp.status().is_success() => Some(url.clone()),
                Ok(resp) if resp.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED => {
                    // fallback to GET
                    match c.get(url.clone()).send().await {
                        Ok(r) if r.status().is_success() => Some(url.clone()),
                        _ => None,
                    }
                }
                _ => None,
            }
        });
    }
    let mut good = Vec::new();
    while let Some(res) = futs.next().await {
        if let Some(u) = res { good.push(u) }
    }
    good
}

/// Extract likely chunk IDs from webpack runtime code.
/// Looks for patterns like `webpackChunk.push(["123", ...])` or array assignments.
/// Returns unique strings that appear to be chunk identifiers (numbers or hashes).
pub fn extract_chunk_ids(js: &str) -> Vec<String> {
    static CHUNK_PUSH_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"webpackChunk(?:_\w+)?\.push\(\[\["?([\w-]+)"?,"#).unwrap()
    });

    static CHUNK_ARRAY_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"var installedChunks = \{([\s\S]*?)\};"#).unwrap()
    });

    let mut ids = Vec::new();
    let mut seen = HashSet::new();

    // First pattern: push calls
    for caps in CHUNK_PUSH_RE.captures_iter(js) {
        if let Some(id) = caps.get(1) {
            let s = id.as_str().to_string();
            if seen.insert(s.clone()) {
                ids.push(s);
            }
        }
    }

    // Second pattern: hardcoded cases like "7561 === e ? "static/chunks/7561-be856e985935a49b.js""
    static HARDCODED_CASE_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(\d+)\s*===\s*e\s*\?\s*"static/chunks/(\d+)-[^"]+\.js""#).unwrap()
    });

    for caps in HARDCODED_CASE_RE.captures_iter(js) {
        if let Some(id) = caps.get(1) {
            let chunk_id = id.as_str().to_string();
            if seen.insert(chunk_id.clone()) {
                ids.push(chunk_id);
            }
        }
    }

    // Third pattern: object literal maps in complex expressions
    // Look for both maps in the pattern: {1255: "7d0bf13e", ...}[e] || e) + "." + {43: "7fa619f5d693091a", ...}[e]
    static COMPLEX_MAP_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"static/chunks/"\+\({([^}]+)}\[e\]\|\|e\)\+"\."\+{([^}]+)}\[e\]"#).unwrap()
    });

    if let Some(caps) = COMPLEX_MAP_RE.captures(js) {
        let map1_raw = caps.get(1).map_or("", |m| m.as_str());
        let map2_raw = caps.get(2).map_or("", |m| m.as_str());

        let entry_re = Regex::new(r#"(\d+):\s*"[\w]+"#).unwrap();

        // Extract chunk IDs from both maps
        for entry in entry_re.captures_iter(map1_raw) {
            if let Some(id) = entry.get(1) {
                let chunk_id = id.as_str().to_string();
                if seen.insert(chunk_id.clone()) {
                    ids.push(chunk_id);
                }
            }
        }

        for entry in entry_re.captures_iter(map2_raw) {
            if let Some(id) = entry.get(1) {
                let chunk_id = id.as_str().to_string();
                if seen.insert(chunk_id.clone()) {
                    ids.push(chunk_id);
                }
            }
        }
    }

    ids
}

/// Extract literal chunk paths such as "static/chunks/1234-abcd.js" appearing in runtime.
pub fn extract_literal_chunk_paths(js: &str) -> Vec<String> {
    static LIT_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"static/chunks/[^"]+?\.js"#).unwrap()
    });
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for mat in LIT_RE.find_iter(js) {
        let s = mat.as_str().to_string();
        if seen.insert(s.clone()) {
            paths.push(s);
        }
    }
    paths
}

/// Generate full chunk URLs using extracted components.
/// Requires the template, optional public path base, and list of chunk IDs.
/// Returns Vec<Url> of constructed URLs (unvalidated).
pub fn generate_chunk_urls(base: Option<&Url>, template: &ChunkFilenameTemplate, chunk_ids: &[String]) -> Vec<Url> {
    let mut urls = Vec::with_capacity(chunk_ids.len());

    for id in chunk_ids {
        if let Some(u) = build_chunk_url(base, template, id) {
            urls.push(u);
        }
    }

    urls
}

/// Mapping-based chunk info extracted from runtime for Next.js style.
pub struct ChunkMapInfo {
    pub prefix: String,
    pub separator: String, // "." or "-"
    pub map_first: std::collections::HashMap<String, String>,
    pub map_second: std::collections::HashMap<String, String>,
}

/// Try to extract chunk map info (Next.js pattern) from runtime.
pub fn extract_chunk_maps(js: &str) -> Option<ChunkMapInfo> {
    // Look for the pattern: "static/chunks/" + ({...}[e] || e) + "." + {...}[e] + ".js"
    static MAP_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#""(static/chunks/)"[^+]*\+[^{]*\{([^}]*)\}\[e\][^+]*\+\s*"\."\s*\+[^{]*\{([^}]*)\}\[e\]"#).unwrap()
    });

    let caps = MAP_RE.captures(js)?;
    let prefix = caps.get(1)?.as_str().to_string();
    let map1_raw = caps.get(2)?.as_str();
    let map2_raw = caps.get(3)?.as_str();

    fn parse_obj(raw: &str) -> std::collections::HashMap<String, String> {
        let mut hm = std::collections::HashMap::new();
        // Convert to JSON by wrapping with braces and quoting keys/values properly
        // naive: split by commas "id: \"hash\""
        for part in raw.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() { continue; }
            let pieces: Vec<&str> = trimmed.split(':').collect();
            if pieces.len() != 2 { continue; }
            let key = pieces[0].trim().trim_matches('"').trim();
            let key = key.to_string();
            let val = pieces[1].trim().trim_matches('"');
            hm.insert(key, val.to_string());
        }
        hm
    }

    Some(ChunkMapInfo {
        prefix,
        separator: ".".to_string(), // Default separator
        map_first: parse_obj(map1_raw),
        map_second: parse_obj(map2_raw),
    })
}

/// Generate chunk URLs using map info and chunk ids.
pub fn generate_urls_from_chunk_maps(base: &Url, maps: &ChunkMapInfo, chunk_ids: &[String]) -> Vec<Url> {
    let mut urls = Vec::new();
    for id in chunk_ids {
        let first = maps.map_first.get(id).cloned().unwrap_or_else(|| id.clone());
        if let Some(second) = maps.map_second.get(id) {
            let path = format!("{}{}{}{}{}.js", maps.prefix, first, maps.separator, second, "");
            if let Ok(u) = base.join(&path) {
                urls.push(u);
            }
        }
    }
    urls
}

/// Extract asset paths (e.g., static/chunks/*.js or static/css/*.css) from a Next.js _buildManifest.js script.
/// Returns a deduplicated Vec<String> of all matched paths.
pub fn extract_paths_from_build_manifest(js: &str) -> Vec<String> {
    static PATH_RE: Lazy<Regex> = Lazy::new(|| {
        // Match "static/chunks/... .js" or "static/css/... .css" inside quotes
        Regex::new(r#"(?:static/(?:chunks|css)/[^"']+?\.(?:js|css))"#).unwrap()
    });

    let mut paths = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for cap in PATH_RE.captures_iter(js) {
        if let Some(m) = cap.get(0) {
            let p = m.as_str().trim_matches('"');
            if seen.insert(p.to_string()) {
                paths.push(p.to_string());
            }
        }
    }

    paths
}

/// Parse the JS source using SWC and print top-level module items (for dev/testing)
pub fn swc_print_top_level(js: &str) {
    let cm: Lrc<SourceMap> = Default::default();
    let js_owned = js.to_owned();
    let fm = cm.new_source_file(FileName::Custom("runtime.js".into()).into(), js_owned);
    let mut parser = Parser::new(Syntax::Es(Default::default()), StringInput::from(&*fm), None);
    match parser.parse_module() {
        Ok(module) => {
            println!("SWC parsed module with {} top-level items", module.body.len());
            for item in &module.body {
                println!("  - {:?}", item);
            }
        }
        Err(e) => {
            println!("SWC parse error: {:?}", e);
        }
    }
} 