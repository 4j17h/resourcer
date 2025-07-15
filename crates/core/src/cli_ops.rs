use crate::{find_sourcemap_urls, parse_sourcemap, sources_list, reconstruct_paths, fetch};
use std::path::Path;
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum CLIError {
    #[error(transparent)]
    Fetch(#[from] crate::FetchError),
    #[error("invalid url: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

/// Async helper to write JS, download sourcemaps, and reconstruct sources.
pub async fn save_js_and_sources(body: &str, url_str: &str, out_root: &Path) -> Result<(), CLIError> {
    let parsed = Url::parse(url_str)?;
    let rel_path = parsed.path().trim_start_matches('/');
    let dest_path = out_root.join(rel_path);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&dest_path, body)?;

    // Detect sourcemap URLs
    let base_url = &parsed;
    let map_urls = find_sourcemap_urls(base_url, body);
    for mu in map_urls {
        // Fetch sourcemap (allow local join)
        let map_text_opt = if mu.scheme() == "file" {
            mu.to_file_path().ok().and_then(|p| std::fs::read_to_string(&p).ok())
        } else {
            fetch::fetch_with_retries(mu.as_str(), 3).await.ok()
        };

        let map_str = match map_text_opt { Some(s) => s, None => continue };

        // Write sourcemap file alongside js
        let map_dest = dest_path.with_extension("js.map");
        if let Some(parent) = map_dest.parent() { std::fs::create_dir_all(parent)?; }
        std::fs::write(&map_dest, &map_str)?;

        if let Ok(sm) = parse_sourcemap(&map_str) {
            let paths = reconstruct_paths(out_root, &sm);
            for (idx, p) in paths.iter().enumerate() {
                if let Some(content) = sm.get_source_contents(idx as u32) {
                    if let Some(parent) = p.parent() { std::fs::create_dir_all(parent)?; }
                    std::fs::write(p, content)?;
                }
            }
        }
    }
    Ok(())
}

/// Handle the list-urls command logic
pub async fn handle_list_urls(input: &str, json: bool, show_sources: bool) -> Result<(), CLIError> {
    if !std::path::Path::new(input).is_file() {
        return Err(CLIError::Other(format!("input file '{}' does not exist or is not a file", input)));
    }
    
    // Read file content
    let js = std::fs::read_to_string(input)?;

    // Convert local path to a file:// URL
    let base_path = match std::fs::canonicalize(input) {
        Ok(p) => p,
        Err(_) => std::path::PathBuf::from(input),
    };
    let base = url::Url::from_file_path(&base_path)
        .map_err(|_| CLIError::Other("failed to construct file:// URL from path".to_string()))?;
    
    let urls = find_sourcemap_urls(&base, &js);
    
    if json {
        println!("{}", serde_json::to_string_pretty(&urls).map_err(|e| CLIError::Other(e.to_string()))?);
    } else {
        for u in &urls {
            println!("{}", u);
        }
    }

    if show_sources {
        for u in &urls {
            if u.scheme() == "file" {
                if let Ok(path) = u.to_file_path() {
                    if let Ok(map_contents) = std::fs::read_to_string(&path) {
                        match parse_sourcemap(&map_contents) {
                            Ok(sm) => {
                                println!("\n# sources in {}", path.display());
                                for s in sources_list(&sm) {
                                    println!("  {s}");
                                }
                            }
                            Err(e) => eprintln!("failed to parse sourcemap {}: {e}", path.display()),
                        }
                    } else {
                        eprintln!("could not read map file {}", path.display());
                    }
                }
            }
        }
    }
    
    Ok(())
} 