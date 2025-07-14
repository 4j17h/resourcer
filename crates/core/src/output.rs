use std::path::Path;
use std::fs;
use std::io::{self, ErrorKind};

/// Ensure the given output directory exists. Creates missing parent directories as needed.
///
/// Returns Ok(()) if the directory already exists (and is a directory) or is successfully created.
/// Returns an io::Error if creation fails or if a non-directory entity exists at the path.
pub fn ensure_output_dir<P: AsRef<Path>>(output_path: P) -> io::Result<()> {
    let path = output_path.as_ref();
    if path.exists() {
        // If exists but is not a directory, return error.
        if !path.is_dir() {
            return Err(io::Error::new(ErrorKind::AlreadyExists, "Output path exists but is not a directory"));
        }
        return Ok(());
    }
    fs::create_dir_all(path)
} 