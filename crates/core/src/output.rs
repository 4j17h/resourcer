use std::path::Path;
use std::fs;
use std::io::{self, ErrorKind};
use walkdir::WalkDir;
use sha2::{Sha256, Digest};

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

/// Mirror directory structure from src_root into dst_root without copying files.
pub fn mirror_structure(src_root: &Path, dst_root: &Path) -> io::Result<()> {
    for entry in WalkDir::new(src_root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_dir() {
            let relative = entry.path().strip_prefix(src_root).unwrap();
            let dst_dir = dst_root.join(relative);
            if !dst_dir.exists() {
                fs::create_dir_all(&dst_dir)?;
            }
        }
    }
    Ok(())
}

/// Copy all files from src_root to dst_root, mirroring relative paths.
pub fn copy_files(src_root: &Path, dst_root: &Path) -> io::Result<()> {
    for entry in WalkDir::new(src_root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(src_root).unwrap();
            let dst_path = dst_root.join(relative);
            if let Some(parent) = dst_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

fn file_hash(path: &Path) -> io::Result<Vec<u8>> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hasher.finalize().to_vec())
}

/// Validate that dst_root contains identical files to src_root. Returns list of mismatched relative paths.
pub fn validate_output(src_root: &Path, dst_root: &Path) -> io::Result<Vec<String>> {
    let mut mismatches = Vec::new();
    for entry in WalkDir::new(src_root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(src_root).unwrap();
            let dst_path = dst_root.join(rel);
            if !dst_path.exists() {
                mismatches.push(rel.to_string_lossy().to_string());
                continue;
            }
            let src_hash = file_hash(entry.path())?;
            let dst_hash = file_hash(&dst_path)?;
            if src_hash != dst_hash {
                mismatches.push(rel.to_string_lossy().to_string());
            }
        }
    }
    Ok(mismatches)
} 