# Sourcedumper

Sourcedumper is a high-performance CLI tool that automatically reconstructs the original source tree of Web-packed (or otherwise bundled) JavaScript applications.  It crawls a target URL (or local file), discovers sourcemaps and runtime chunk mappings, then rebuilds the project structure on disk – perfect for security auditors, reverse-engineers and developers.

---

## Features (road-map)

* Fast Rust core for HTML fetching & parsing
* Automatic sourcemap discovery and reconstruction
* Webpack runtime parser (Vite / Parcel planned via plugin system)
* Optional Playwright-powered SPA analysis (`--browser` flag)
* Dry-run and URL-listing modes
* Resource limits for large sites
* Cross-platform pre-compiled binaries
* Extensible architecture for new runtime detectors

---

## Repository layout

```
crates/
  core/        # Rust library – extraction engine
  cli/         # Rust binary – main entry point (depends on `core`)
  detectors/   # (planned) plugin crate for additional runtimes

tools/
  browser-harness/ # Node package with Playwright scripts for SPA mode

docs/         # Project documentation
prd.md         # High-level product requirements document
```

---

## Building from source

Prerequisites:

* Rust toolchain (`rustup install stable`) – stable channel (see `.rust-toolchain.toml`)
* PNPM ≥ 8 (`npm install -g pnpm`) ‑ for Node helper packages

Steps:

```bash
# clone and build
git clone https://github.com/your-org/sourcedumper.git
cd sourcedumper
cargo build --release     # builds all workspace crates

# (optional) install Playwright deps for browser mode
pnpm install --frozen-lockfile
pnpm --filter browser-harness exec playwright install
```

Run the CLI:
```bash
./target/release/sourcedumper --help
```

---

## Contributing

See `CODE_OF_CONDUCT.md` and feel free to open issues or pull requests.  All contributions are welcome!

---

## License

This project is licensed under the MIT License – see `LICENSE` for details. 