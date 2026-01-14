//! Filesystem operations with safety checks

use crate::core::errors::{MoleError, Result};
use std::path::Path;
use walkdir::WalkDir;

/// Calculate the size of a directory recursively
pub fn dir_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let mut total = 0u64;

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }

    Ok(total)
}

/// Format bytes into human-readable string
pub fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}

/// Check if we have permission to delete a path
pub fn can_delete(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    // Check if we can write to parent directory
    if let Some(parent) = path.parent() {
        if let Ok(metadata) = std::fs::metadata(parent) {
            return !metadata.permissions().readonly();
        }
    }

    false
}

/// Check if running as root
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Safely delete a file or directory
pub fn safe_delete(path: &Path, dry_run: bool) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let size = dir_size(path)?;

    if dry_run {
        return Ok(size);
    }

    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                MoleError::PermissionDenied {
                    path: path.display().to_string(),
                }
            } else {
                MoleError::Io(e)
            }
        })?;
    } else {
        std::fs::remove_file(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                MoleError::PermissionDenied {
                    path: path.display().to_string(),
                }
            } else {
                MoleError::Io(e)
            }
        })?;
    }

    Ok(size)
}

/// Delete contents of a directory but keep the directory itself
pub fn clean_directory(path: &Path, dry_run: bool) -> Result<u64> {
    if !path.exists() || !path.is_dir() {
        return Ok(0);
    }

    let mut total_freed = 0u64;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        total_freed += safe_delete(&entry_path, dry_run)?;
    }

    Ok(total_freed)
}

/// Count files in a directory
pub fn count_files(path: &Path) -> usize {
    if !path.exists() {
        return 0;
    }

    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count()
}
