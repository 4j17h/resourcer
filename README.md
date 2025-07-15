# ReSourceR

ReSourceR is a high-performance Rust CLI tool that automatically reconstructs the original source tree of Web-packed (or otherwise bundled) JavaScript applications.  It crawls a target URL (or local file), discovers sourcemaps and runtime chunk mappings, then rebuilds the project structure on disk – perfect for security auditors, reverse-engineers and developers.

---

## Features

* Fast Rust core for HTML fetching & parsing (includes `cli_ops.rs` for CLI operations and `url_utils.rs` for URL handling)
* Automatic sourcemap discovery and reconstruction
* Webpack runtime parser (Vite / Parcel planned via plugin system)
* Dry-run and URL-listing modes
* Resource limits for large sites
* Cross-platform pre-compiled binaries
* Extensible architecture for new runtime detectors (planned via plugin crate)

---

## Repository layout

```
crates/
  core/        # Rust library – extraction engine
  cli/         # Rust binary – main entry point (depends on `core`)
  detectors/   # Plugin crate for additional runtimes (planned)

tools/
  browser-harness/ # Node package with Playwright scripts for SPA mode (currently empty)

docs/         # Project documentation
```

---

## How it Works

ReSourceR employs a multi-stage analysis and reconstruction pipeline to reverse-engineer bundled JavaScript applications:

### 1. Initial Discovery Phase

**HTML Fetching & Script Extraction**
- Fetches the target webpage using a robust HTTP client with retry logic and exponential backoff
- Parses HTML to identify all `<script>` tags and extract JavaScript URLs
- Validates and resolves relative URLs against the base page URL

**Runtime Detection**
- Scans discovered scripts for Webpack runtime patterns (e.g., `/webpack-*.js`)
- Looks for build manifest files (`_buildManifest.js` or `buildManifest.js`)
- Uses regex patterns to identify framework-specific bundler signatures

### 2. Bundle Analysis Phase

**Webpack Runtime Parsing**
- Extracts chunk filename templates using multiple regex strategies:
  - Anonymous function form: `__webpack_require__.u = function(chunkId) { return "prefix" + chunkId + "suffix"; }`
  - Arrow function form: `__webpack_require__.u = (chunkId) => "prefix" + chunkId + "suffix"`
  - Template literal form: `return \`prefix${chunkId}suffix\``
- Discovers public path configuration (`__webpack_require__.p`)
- Parses chunk mapping objects for Next.js-style builds

**Chunk Discovery Strategies**
- **Build Manifest Method**: Parses `_buildManifest.js` to extract all asset paths directly
- **Runtime Inference**: Uses extracted templates and chunk IDs to generate probable URLs
- **Pattern Matching**: Finds literal chunk paths embedded in runtime code
- **Map-based Construction**: Handles complex chunk naming schemes with hash mappings

### 3. Asset Enumeration Phase

**Chunk URL Generation**
- Combines discovered templates with extracted chunk IDs
- Handles various naming patterns (numeric IDs, content hashes, hybrid schemes)
- Resolves URLs against proper base paths (public path or derived from runtime location)

**Concurrent Validation**
- Performs HTTP HEAD requests to verify chunk URLs exist
- Falls back to GET requests if HEAD is not allowed
- Implements configurable concurrency limits to avoid overwhelming servers
- Filters out non-existent or inaccessible resources

### 4. Sourcemap Processing Phase

**Sourcemap Discovery**
- Scans JavaScript files for sourcemap comments:
  - Single-line: `//# sourceMappingURL=map.js.map`
  - Multi-line: `/*# sourceMappingURL=map.js.map */`
- Validates and resolves sourcemap URLs against base paths
- Deduplicates discovered sourcemap references

**Sourcemap Parsing & Validation**
- Downloads and parses sourcemap JSON using the `sourcemap` crate
- Extracts original source file paths from the `sources` array
- Validates sourcemap structure and handles malformed maps gracefully

### 5. Source Reconstruction Phase

**Content Extraction**
- **Direct Method**: Uses `sourcesContent` array when available in sourcemaps
- **SWC-based Method**: When `sourcesContent` is missing:
  - Parses generated JavaScript using SWC (Speedy Web Compiler)
  - Builds AST and maps generated code positions to original sources
  - Extracts code segments using sourcemap token mappings
  - Reconstructs original files by concatenating mapped segments

**File System Output**
- Creates directory structure matching original source tree
- Writes reconstructed files with proper extensions and content
- Provides detailed logging of reconstruction progress and statistics

### 6. CLI Modes & Features

**List URLs Mode** (`list-urls`)
- Quick analysis mode that only discovers and lists sourcemap URLs
- Optional JSON output for programmatic consumption
- Can display source lists from local sourcemap files

**Dump Mode** (`dump`)
- Full reconstruction pipeline with configurable options:
  - **Dry-run**: Analysis only, no file writing
  - **Concurrency control**: Adjustable concurrent download limits
  - **Resource limits**: Maximum file count restrictions
  - **Output directory**: Customizable reconstruction target

**Error Handling & Resilience**
- Comprehensive error handling for network failures, parsing errors, and file system issues
- Graceful degradation when partial information is available
- Detailed logging at multiple verbosity levels
- Timeout protection and resource limit enforcement

This architecture allows ReSourceR to handle a wide variety of bundler configurations and deployment scenarios while maintaining high performance through concurrent operations and intelligent fallback strategies.

---

## Building from source

Prerequisites:

* Rust toolchain (`rustup install stable`) – stable channel (see `.rust-toolchain.toml`)

Steps:

```bash
# clone and build
git clone https://github.com/4j17h/resourcer.git
cd resourcer
cargo build --release     # builds all workspace crates
```

Run the CLI:
```bash
./target/release/resourcer_cli --help
```

---

## Contributing

See `CODE_OF_CONDUCT.md` and feel free to open issues or pull requests.  All contributions are welcome!

---

## License

This project is licensed under the MIT License – see `LICENSE` for details. 