use clap::{Parser, Subcommand, ArgAction};
use resourcer_core::*;
use std::{collections::HashSet, path::PathBuf};
use regex::Regex;
use url::Url;

#[derive(Subcommand)]
enum Commands {
    #[command(
        about = "Point ReSourceR at any Webpack-powered page and instantly list its sources and sourcemap links",
        long_about = "Give ReSourceR a path or URL to a Webpack (or other bundler) JavaScript file and it will scan for \"//# sourceMappingURL=...\" or \"/*# sourceMappingURL=... */\" comments, printing every .map it finds. Perfect for a quick peek before a full dump.\n\nExamples:\n  resourcer_cli list-urls --input ./dist/main.js\n  resourcer_cli list-urls --input ./dist/main.js --json"
    )]
    ListUrls {
        /// Input JavaScript file to analyze
        #[arg(long, value_name = "FILE")]
        input: String,
        /// Output list as pretty-printed JSON
        #[arg(long, help = "Print JSON array instead of plain text")]
        json: bool,
        /// Also load located sourcemap files (local only) and print their `sources[]` entries
        #[arg(long, help = "For each discovered .map file that is on disk, print its sources list")]
        show_sources: bool,
    },
    #[command(
        about = "Give ReSourceR any Webpack-powered page & it will autodiscover all chunks/sourcemaps and rebuild the source",
        long_about = "Just point ReSourceR at a public URL of a site built with Webpack/Next.js/etc. It fetches the HTML, follows the runtime `webpack*.js` & `_buildManifest.js`, enumerates every JS chunk & sourcemap it can find, downloads them (concurrent by default), then reconstructs the original source tree ‑ ready for review, auditing, or diffing.  A local bundle file can also be provided via `--input`."
    )]
    Dump {
        /// Target URL to fetch (mutually exclusive with --input)
        #[arg(long, conflicts_with = "input", value_name = "URL")]
        url: Option<String>,
        /// Local JavaScript file to process
        #[arg(long, conflicts_with = "url", value_name = "FILE")]
        input: Option<String>,
        /// Output directory for reconstructed sources
        #[arg(long, value_name = "DIR", default_value = "out")]
        out: String,
        /// Perform detection only; do not download or write files
        #[arg(long, help = "Detect and list actions without writing files")]
        dry_run: bool,

        /// Maximum number of concurrent downloads (1 = sequential)
        #[arg(long, value_name = "N", default_value = "8")]
        concurrency: usize,

        /// Limit total number of chunk files to download (useful for testing)
        #[arg(long, value_name = "N")]
        max_files: Option<usize>,
    },
    #[command(
        about = "Analyze Single-Page Apps using headless browser (placeholder)",
        long_about = "Use a headless browser to render an SPA so that dynamically injected script tags are captured. This feature corresponds to task 9 and is not yet implemented."
    )]
    Browser {
        #[arg(long, value_name = "URL")]
        url: String,
    },
}

#[derive(Parser)]
#[command(version, about = "ReSourceR – Rust-powered tool to extract bundles, resolve sourcemaps, and reconstruct original source code from minified JavaScript")]
struct Cli {
    /// Increase output verbosity (-v, -vv, etc.)
    #[arg(short, long, global = true, action = ArgAction::Count)]
    verbose: u8,

    /// Silence all output except errors
    #[arg(short, long, global = true, action = ArgAction::SetTrue)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

fn init_logging(verbosity: u8, quiet: bool) {
    use log::LevelFilter::*;
    let level = if quiet {
        Error
    } else {
        match verbosity {
            0 => Info,
            1 => Debug,
            _ => Trace,
        }
    };
    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp(None)
        .init();
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_logging(cli.verbose, cli.quiet);

    match cli.command {
        Commands::ListUrls { input, json, show_sources } => {
            if let Err(e) = handle_list_urls(&input, json, show_sources).await {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Dump { url, input, out, dry_run, concurrency, max_files } => {
            if let Some(page_url) = url {
                if let Err(e) = run_dump_remote(&page_url, &out, dry_run, concurrency, max_files).await {
                    eprintln!("error: {:?}", e);
                    std::process::exit(1);
                }
            } else if let Some(local) = input {
                eprintln!("local dump not yet implemented: {}", local);
                std::process::exit(1);
            } else {
                eprintln!("error: either --url or --input must be provided");
                std::process::exit(1);
            }
        }
        Commands::Browser { .. } => {
            eprintln!("browser command not yet implemented");
            std::process::exit(1);
        }
    }
}

async fn run_dump_remote(page_url: &str, _out_dir: &str, dry_run: bool, concurrency: usize, max_files: Option<usize>) -> Result<(), CLIError> {
    println!("Fetching HTML from {}", page_url);
    let html = fetch_html(page_url).await?;
    let base = Url::parse(page_url)?;

    // Extract script URLs using core function
    let script_urls = extract_script_urls(&html, &base);
    
    // Look for the Webpack runtime
    let runtime_re = Regex::new(r"/webpack-[^/]+\.js(?:\?.*)?$").unwrap();
    let runtime_url = script_urls.iter().find(|u| runtime_re.is_match(u.path()));

    let runtime_url = match runtime_url {
        Some(u) => {
            println!("Identified Webpack runtime: {}", u);
            u.clone()
        }
        None => {
            println!("No Webpack runtime script found matching pattern /webpack-*.js");
            return Ok(());
        }
    };

    println!("Fetching runtime content...");
    let runtime_js = fetch_html(runtime_url.as_str()).await?;

    // Log if sourcemap is enabled in the runtime JS using core function
    if let Some(sm_url) = find_sourcemap_url_in_js(&runtime_js) {
        println!("Sourcemap enabled in runtime: {}", sm_url);
    } else {
        println!("No sourcemap comment found in runtime JS");
    }

    // Determine chunk filename template (fallback) and helpers
    let template_opt = infer_chunk_filename_template(&runtime_js);

    // First attempt: use _buildManifest.js to list all asset paths
    let manifest_re = Regex::new(r"/_?buildManifest\.js$",).unwrap();
    let manifest_url_opt = script_urls.iter().find(|u| manifest_re.is_match(u.path()));

    let chunk_urls: Vec<Url> = if let Some(manifest_url) = manifest_url_opt {
        println!("Found build manifest script: {}", manifest_url);
        let manifest_js = fetch_html(manifest_url.as_str()).await?;

        // Extract asset paths (JS & CSS) from manifest
        let paths = extract_paths_from_build_manifest(&manifest_js);
        println!("Found {} asset paths in build manifest", paths.len());

        // Derive base using runtime url (static prefix) - using core function
        let base_chunks = derive_base_from_runtime(&runtime_url, "static/");

        let mut urls = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for p in paths {
            // We only care about .js files for now
            if !p.ends_with(".js") { continue; }
            if let Ok(u) = base_chunks.join(&p) {
                if seen.insert(u.as_str().to_string()) {
                    urls.push(u);
                }
            }
        }

        urls
    } else {
        // If manifest not found, fallback to runtime-based inference as before

        // Determine public path base URL
        let public_path = extract_public_path(&runtime_js);
        let base_for_chunks: Option<url::Url> = match public_path.as_ref() {
            Some(pp) => {
                // __webpack_require__.p provided (absolute or relative)
                if let Ok(abs) = url::Url::parse(pp) {
                    Some(abs)
                } else if let Ok(joined) = base.join(pp) {
                    Some(joined)
                } else {
                    None
                }
            }
            None => {
                // No explicit public path – default to origin; path trimming handled later
                Some(base.clone())
            }
        };

        let has_public_path = public_path.is_some();

        if let Some(bu) = &base_for_chunks {
            println!("Public path base: {}", bu);
        } else {
            println!("No public path detected; chunk URLs may be relative.");
        }

        if let Some(template) = template_opt {
            println!("Chunk filename template: prefix='{}' suffix='{}'", template.prefix, template.suffix);
            let chunk_ids = extract_chunk_ids(&runtime_js);
            println!("Found {} chunk IDs", chunk_ids.len());

            let dyn_base = if has_public_path {
                base_for_chunks.as_ref().cloned()
            } else {
                Some(derive_base_from_runtime(&runtime_url, &template.prefix))
            };

            generate_chunk_urls(dyn_base.as_ref(), &template, &chunk_ids)
        } else if let Some(map_info) = extract_chunk_maps(&runtime_js) {
            println!("Using map-based filename construction (separator '{}')", map_info.separator);
            let chunk_ids = extract_chunk_ids(&runtime_js);
            println!("Found {} chunk IDs", chunk_ids.len());

            let dyn_base = if has_public_path {
                base_for_chunks.as_ref().cloned().unwrap_or(base.clone())
            } else {
                derive_base_from_runtime(&runtime_url, &map_info.prefix)
            };

            generate_urls_from_chunk_maps(&dyn_base, &map_info, &chunk_ids)
        } else {
            println!("Could not infer template or maps; falling back to literal path extraction.");
            let paths = extract_literal_chunk_paths(&runtime_js);
            println!("Found {} literal chunk paths", paths.len());
            let mut urls = Vec::new();

            // Use derive_base_from_runtime with "static/chunks/" prefix for literal paths
            let literal_base = if has_public_path {
                base_for_chunks.as_ref().cloned().unwrap_or(base.clone())
            } else {
                derive_base_from_runtime(&runtime_url, "static/chunks/")
            };

            for p in paths {
                if let Ok(u) = literal_base.join(&p) {
                    urls.push(u);
                }
            }
            urls
        }
    };

    println!("Generated {} chunk URLs", chunk_urls.len());

    if dry_run {
        for u in &chunk_urls {
            println!("  - {}", u);
        }
        println!("Dry run complete. No downloads performed.");
        return Ok(());
    }

    println!("Validating chunk URLs...");
    let live_urls = validate_chunk_urls(chunk_urls).await;
    println!("{} URLs responded with 2xx", live_urls.len());

    let mut live_urls = live_urls; // make mutable
    if let Some(max_n) = max_files {
        if live_urls.len() > max_n {
            live_urls.truncate(max_n);
            println!("Truncated to {} URLs due to --max-files", max_n);
        }
    }

    if live_urls.is_empty() {
        println!("No downloadable chunk URLs found – exiting.");
        return Ok(());
    }

    // Prepare output directory
    let out_root: PathBuf = if _out_dir == "out" {
        // Use "out/<host>" when user did not specify --out
        let host = base.host_str().unwrap_or("site");
        PathBuf::from("out").join(host)
    } else {
        PathBuf::from(_out_dir)
    };

    ensure_output_dir(&out_root)?;

    println!("Downloading {} chunk files to {:?} ...", live_urls.len(), out_root);

    if concurrency <= 1 {
        // Sequential download
        for (idx, u) in live_urls.iter().enumerate() {
            println!("[{}/{}] downloading {}", idx + 1, live_urls.len(), u);
            match resourcer_core::fetch::fetch_with_retries(u.as_str(), 3).await {
                Ok(body) => {
                    if let Err(e) = save_js_and_sources(&body, u.as_str(), &out_root).await {
                        eprintln!("✖ error processing {}: {:?}", u, e);
                    } else {
                        println!("✔ saved {}", u);
                    }
                }
                Err(e) => eprintln!("✖ failed to download {}: {:?}", u, e),
            }
        }
    } else {
        // Parallel via download_many
        let url_strings: Vec<String> = live_urls.iter().map(|u| u.to_string()).collect();
        let cfg = DownloadManagerConfig { concurrency, ..Default::default() };
        let results = download_many(url_strings, cfg).await;

        for (idx, res) in results.into_iter().enumerate() {
            match res.content {
                Some(body) => {
                    if let Err(e) = save_js_and_sources(&body, &res.url, &out_root).await {
                        eprintln!("✖ error processing {}: {:?}", res.url, e);
                    } else {
                        println!("[{}] ✔ saved {}", idx + 1, res.url);
                    }
                }
                None => eprintln!("✖ failed {}: {:?}", res.url, res.error),
            }
        }
    }

    println!("All downloads and source reconstruction complete. Output at {:?}", out_root);

    Ok(())
}
